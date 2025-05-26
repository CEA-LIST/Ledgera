/**************************************************************************************************
 * Copyright (c) 2025 CEA (Commissariat à l'énergie atomique et aux énergies alternatives)
 *   contributors:
 *   - Erwan Mahe ( erwan.mahe@cea.fr )
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *       https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 *************************************************************************************************/

use ledgera_comms::{comm_session::PubSubNetwork, error::LedgeraCommunicationError};
use ledgera_core_logic::{roles::LedgeraCoreRoles, topics::LedgeraCorePublicationTopics};
use ledgera_pki::manager::PublicKeyInfrastructure;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::messages::{
    deliver::LedgeraTransactionDeliveryNotification, nres::LedgeraComputationResultNotification,
};
use ledgera_types::requests::rfun::LedgeraRequestFunctionInstanceProposal;

use crate::client_logic::client_state::LedgeraClientNodeState;
use crate::client_logic::handle_core_msgs::client_handling_of_core_messages;
use crate::comms::feedback_from_core_client::ValidatedCoreFeedbackMessage;
use crate::runtime::runtime_io::CoreClientRuntime;

pub struct LedgeraClientRunOutput<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
> {
    pub core_runtime: CoreClientRuntime<PKI, Sess, LAT>,
    pub to_app_stream_of_validated_core_msgs:
        tokio::sync::mpsc::Receiver<ValidatedCoreFeedbackMessage<LAT>>,
}

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork, LAT: LedgeraApplicationTemplate>
    LedgeraClientRunOutput<PKI, Sess, LAT>
{
    pub fn new(
        core_runtime: CoreClientRuntime<PKI, Sess, LAT>,
        to_app_stream_of_validated_core_msgs: tokio::sync::mpsc::Receiver<
            ValidatedCoreFeedbackMessage<LAT>,
        >,
    ) -> Self {
        Self {
            core_runtime,
            to_app_stream_of_validated_core_msgs,
        }
    }
}

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork, LAT: LedgeraApplicationTemplate>
    LedgeraClientNodeState<PKI, Sess, LAT>
{
    /// runs the client and returns a sender to submit user requests
    pub async fn run(
        &mut self,
    ) -> Result<
        LedgeraClientRunOutput<PKI, Sess, LAT>,
        LedgeraCommunicationError<Sess::CommRuntimeError>,
    > {
        let name_as_client = hex::encode(PKI::serialize_verifying_key(
            &PKI::get_verifying_key_from_signing_key(&self.comm_params.signing_key),
        ));

        // for now subscribe to rfun to have the specs of computations
        // maybe later dedicated private channels for privacy of comp spec ?
        let (rfun_sender, rfun_receiver) = tokio::sync::mpsc::channel(128);
        // subscribing to delivered transactions and client notifications
        let (delivered_txs_sender, delivered_txs_receiver) = tokio::sync::mpsc::channel(128);
        let (client_notifications_sender, client_notifications_receiver) =
            tokio::sync::mpsc::channel(128);
        {
            let mut comm_sess = self.comm_session.lock().await;
            match comm_sess
                .subscribe_to_topic_and_deserialize_as::<LedgeraRequestFunctionInstanceProposal<LAT>>(
                    &self.comm_params,
                    &LedgeraCorePublicationTopics::Rfun.get_publication_topic_str(self.service.as_ref()),
                    rfun_sender
                ).await
            {
                Ok(_) => {
                    log::info!(
                        "As {:?} : subscribed to Rfun client requests for service '{:}'",
                        LedgeraCoreRoles::Client,
                        self.service.get_service_name()
                    );
                }
                Err(e) => {
                    return Err(e);
                }
            }
            match comm_sess
                .subscribe_to_topic_and_deserialize_as::<LedgeraTransactionDeliveryNotification>(
                    &self.comm_params,
                    &LedgeraCorePublicationTopics::TransactionDelivery
                        .get_publication_topic_str(self.service.as_ref()),
                    delivered_txs_sender,
                )
                .await
            {
                Ok(_) => {
                    log::info!(
                        "As {:?} : subscribed to delivered transactions for service '{:}'",
                        LedgeraCoreRoles::Client,
                        self.service.get_service_name()
                    );
                }
                Err(e) => {
                    return Err(e);
                }
            }
            match comm_sess
                .subscribe_to_topic_and_deserialize_as::<LedgeraComputationResultNotification<LAT::Data>>(
                    &self.comm_params,
                    &LedgeraCorePublicationTopics::Nout(name_as_client).get_publication_topic_str(self.service.as_ref()),
                    client_notifications_sender
                ).await
            {
                Ok(_) => {
                    log::info!(
                        "As {:?} : subscribed to client notifications",
                        LedgeraCoreRoles::Client
                    );
                },
                Err(e) => {
                    return Err(e);
                }
            }
        }
        // ***
        // handling core messages and forwarding those that are validated to user application
        let (validated_core_msgs_sender, validated_core_msgs_receiver) =
            tokio::sync::mpsc::channel(128);
        {
            let comm_params_ref = self.comm_params.clone();
            self.handle_internal_messages_task = Some(tokio::spawn(async move {
                client_handling_of_core_messages::<PKI, LAT>(
                    comm_params_ref,
                    rfun_receiver,
                    delivered_txs_receiver,
                    client_notifications_receiver,
                    validated_core_msgs_sender,
                )
                .await;
            }));
        }

        let runtime = CoreClientRuntime::new(
            self.comm_session.clone(),
            self.comm_params.clone(),
            self.service.clone(),
        );

        Ok(LedgeraClientRunOutput::new(
            runtime,
            validated_core_msgs_receiver,
        ))
    }
}
