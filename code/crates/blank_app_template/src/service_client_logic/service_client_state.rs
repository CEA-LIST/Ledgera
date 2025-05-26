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

use std::sync::Arc;

use ledgera_comms::comm_api::{
    LedgeraInternalCommunicationInterface, LedgeraInternalCommunicationParameters,
};
use ledgera_comms::comm_session::PubSubNetwork;
use ledgera_comms::error::LedgeraCommunicationError;
use ledgera_node_client::comms::feedback_from_core_client::ValidatedCoreFeedbackMessage;
use ledgera_node_client::runtime::runtime_io::CoreClientRuntime;
use ledgera_pki::manager::PublicKeyInfrastructure;

use crate::lat_binding::LedgeraServiceTemplate;
use crate::service_client_logic::{
    behavior::LedgeraServiceClientBehavior, runtime_io::ServiceClientRuntimeIO,
    LedgeraServiceTemplateType1Message, ServicesTemplateDedicatedTopics,
    LEDGERA_SERVICE_CLIENT_ROLE,
};

pub struct LedgeraServiceClientState<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork> {
    comm_session: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    comm_params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
    service: Arc<LedgeraServiceTemplate>,
    //
    main_task: Option<tokio::task::JoinHandle<()>>,
}

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork> LedgeraServiceClientState<PKI, Sess> {
    pub fn new(
        comm_session: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
        comm_params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
        service: Arc<LedgeraServiceTemplate>,
    ) -> Self {
        Self {
            comm_session,
            comm_params,
            service,
            main_task: None,
        }
    }

    /// Runs the service client and returns its IO interface.
    pub async fn run(
        &mut self,
        // the associated core client IO interface
        core_client_runtime_io: CoreClientRuntime<PKI, Sess, LedgeraServiceTemplate>,
        // the stream of validated core messages emitted by the co-located core client
        mut to_app_stream_of_validated_core_msgs: tokio::sync::mpsc::Receiver<
            ValidatedCoreFeedbackMessage<LedgeraServiceTemplate>,
        >,
    ) -> Result<ServiceClientRuntimeIO, LedgeraCommunicationError<Sess::CommRuntimeError>> {
        let name_as_client = hex::encode(PKI::serialize_verifying_key(
            &PKI::get_verifying_key_from_signing_key(&self.comm_params.signing_key),
        ));

        // TODO : subscribe to relevant topics (communication channels)
        // in order to communicate outside Ledgera Core
        let (type1_msgs_sender, mut type1_msgs_receiver) = tokio::sync::mpsc::channel(128);
        {
            let mut comm_sess = self.comm_session.lock().await;
            match comm_sess
                .subscribe_to_topic_and_deserialize_as::<LedgeraServiceTemplateType1Message>(
                    &self.comm_params,
                    &ServicesTemplateDedicatedTopics::PrivateTopic.get_topic_str(&name_as_client),
                    type1_msgs_sender,
                )
                .await
            {
                Ok(_) => {
                    log::info!(
                        "As {:?} : subscribed to a certain topic",
                        LEDGERA_SERVICE_CLIENT_ROLE
                    );
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        // handling user requests
        let (ureq_sender, mut ureq_receiver) = tokio::sync::mpsc::channel(128);
        // forwarding validated core feedback to the TUI so its knowledge can update
        let (core_msgs_sender, core_msgs_receiver_for_tui) = tokio::sync::mpsc::channel(128);
        let comm_session_ref = self.comm_session.clone();
        let comm_params_ref = self.comm_params.clone();
        let service_ref = self.service.clone();
        self.main_task = Some(tokio::spawn(async move {
            let mut behavior = LedgeraServiceClientBehavior::new(
                comm_session_ref,
                comm_params_ref,
                service_ref,
                core_client_runtime_io,
            );
            loop {
                tokio::select! {
                    Some((type1_service_msg, msg_sig)) = type1_msgs_receiver.recv() => {
                        behavior.react_to_service_type1_msg(type1_service_msg, msg_sig).await;
                    },
                    Some(validated_core_msg) = to_app_stream_of_validated_core_msgs.recv() => {
                        behavior.react_to_validated_core_msg(validated_core_msg.clone()).await;
                        // also duplicate core message and send to UI
                        let _ = core_msgs_sender.send(validated_core_msg).await;
                    },
                    Some(user_req) = ureq_receiver.recv() => {
                        behavior.react_to_service_user_req(user_req).await;
                    }
                    // For richer services using Tags for example, you may use a JoinSet of background tasks here.
                }
            }
        }));

        Ok(ServiceClientRuntimeIO {
            user_requests_sender: ureq_sender,
            validated_core_msgs_receiver: core_msgs_receiver_for_tui,
        })
    }
}
