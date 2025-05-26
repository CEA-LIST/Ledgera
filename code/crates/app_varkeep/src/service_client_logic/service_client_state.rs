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

use ledgera_comms::comm_api::{
    LedgeraInternalCommunicationInterface, LedgeraInternalCommunicationParameters,
};
use ledgera_comms::comm_session::PubSubNetwork;
use ledgera_comms::error::LedgeraCommunicationError;
use ledgera_node_client::comms::feedback_from_core_client::ValidatedCoreFeedbackMessage;
use ledgera_node_client::runtime::runtime_io::CoreClientRuntime;
use ledgera_pki::manager::PublicKeyInfrastructure;
use std::sync::Arc;

use crate::lat_binding::LedgeraVarkeepService;
use crate::service_client_logic::behavior::LedgeraServiceClientBehavior;
use crate::service_client_logic::role::LEDGERA_VARKEEP_SERVICE_CLIENT_ROLE;
use crate::service_client_logic::runtime_io::ServiceClientRuntimeIO;
use crate::service_client_logic::service_msgs::messages::LedgeraVarkeepServicePublishLocVarMsg;
use crate::service_client_logic::service_msgs::topics::VarkeepServicesDedicatedTopics;

pub struct LedgeraServiceClientState<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork> {
    comm_session: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    comm_params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
    service: Arc<LedgeraVarkeepService>,
    //
    main_task: Option<tokio::task::JoinHandle<()>>,
}

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork> LedgeraServiceClientState<PKI, Sess> {
    pub fn new(
        comm_session: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
        comm_params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
        service: Arc<LedgeraVarkeepService>,
    ) -> Self {
        Self {
            comm_session,
            comm_params,
            service,
            // ***
            main_task: None,
        }
    }

    /// runs the Service client and returns its Input Output interface
    pub async fn run(
        &mut self,
        // the associated core client runtime
        core_client_runtime_io: CoreClientRuntime<PKI, Sess, LedgeraVarkeepService>,
        // stream of messages from the core
        mut to_app_stream_of_validated_core_msgs: tokio::sync::mpsc::Receiver<
            ValidatedCoreFeedbackMessage<LedgeraVarkeepService>,
        >,
    ) -> Result<ServiceClientRuntimeIO, LedgeraCommunicationError<Sess::CommRuntimeError>> {
        let name_as_client = hex::encode(PKI::serialize_verifying_key(
            &PKI::get_verifying_key_from_signing_key(&self.comm_params.signing_key),
        ));

        // TODO : subscribe to relevant topics (communication channels)
        // in order to communicate outside Ledgera Core
        let (pub_loc_var_msgs_sender, mut pub_loc_var_msgs_receiver) =
            tokio::sync::mpsc::channel(128);
        {
            let mut comm_sess = self.comm_session.lock().await;
            match comm_sess
                .subscribe_to_topic_and_deserialize_as::<LedgeraVarkeepServicePublishLocVarMsg>(
                    &self.comm_params,
                    &VarkeepServicesDedicatedTopics::PublishLocalVariable
                        .get_topic_str(&name_as_client),
                    pub_loc_var_msgs_sender,
                )
                .await
            {
                Ok(_) => {
                    log::info!(
                        "As {:?} : subscribed to a certain topic",
                        LEDGERA_VARKEEP_SERVICE_CLIENT_ROLE
                    );
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        // handling user requests
        let (ureq_sender, mut ureq_receiver) = tokio::sync::mpsc::channel(128);
        let (ui_feed_sender, ui_feed_receiver) = tokio::sync::mpsc::channel(128);
        let comm_session_ref = self.comm_session.clone();
        let comm_params_ref = self.comm_params.clone();
        let service_ref = self.service.clone();
        self.main_task = Some(tokio::spawn(async move {
            let mut behavior = LedgeraServiceClientBehavior::new(
                comm_session_ref,
                comm_params_ref,
                service_ref,
                core_client_runtime_io,
                ui_feed_sender,
            );
            loop {
                tokio::select! {
                    Some((pub_loc_var_msg,msg_sig)) = pub_loc_var_msgs_receiver.recv() => {
                        behavior.react_to_service_msg_publish_local_var(
                            pub_loc_var_msg,
                            msg_sig
                        ).await;
                    },
                    Some(validated_core_msg) = to_app_stream_of_validated_core_msgs.recv() => {
                        behavior.react_to_validated_core_msg(validated_core_msg).await;
                    },
                    Some(user_req) = ureq_receiver.recv() => {
                        behavior.react_to_service_user_req(user_req).await;
                    }
                }
            }
        }));

        Ok(ServiceClientRuntimeIO {
            user_requests_sender: ureq_sender,
            tui_feed_receiver: ui_feed_receiver,
        })
    }
}
