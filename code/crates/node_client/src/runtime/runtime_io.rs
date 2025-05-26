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

use ledgera_comms::{
    comm_api::{LedgeraInternalCommunicationInterface, LedgeraInternalCommunicationParameters},
    comm_session::PubSubNetwork,
};
use ledgera_pki::manager::PublicKeyInfrastructure;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;

pub struct CoreClientRuntime<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
> {
    // pointer to the session, to be able to send requests to the core
    pub(crate) comm_session_ref:
        Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    // pointer to the Ledgent's Ledgera network parameters
    pub(crate) comm_params_ref: Arc<LedgeraInternalCommunicationParameters<PKI>>,
    // pointer to information about the Ledgera Functional Template
    pub(crate) service_ref: Arc<LAT>,
    // TODO: keeps track of ongoing function instances that are flagged as "synchronous"
    //pub(crate)
    // counter to provided unique "nounces" for each new function instance the client proposes
    // so that even if two such functions have the exact same specification,
    // they have different identifiers (signature of the emitted "Rfun" request)
    pub(crate) next_computation_id_when_submitting: std::sync::atomic::AtomicU32,
    // "receiver" part of a MPSC channel used to forward core messages to the application layer
    //pub to_app_stream_of_validated_core_msgs : tokio::sync::mpsc::Receiver<ValidatedCoreFeedbackMessage<LAT>>
}

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork, LAT: LedgeraApplicationTemplate>
    CoreClientRuntime<PKI, Sess, LAT>
{
    pub fn new(
        comm_session_ref: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
        comm_params_ref: Arc<LedgeraInternalCommunicationParameters<PKI>>,
        service_ref: Arc<LAT>,
        //to_app_stream_of_validated_core_msgs: tokio::sync::mpsc::Receiver<ValidatedCoreFeedbackMessage<LAT>>
    ) -> Self {
        Self {
            comm_session_ref,
            comm_params_ref,
            service_ref,
            next_computation_id_when_submitting: std::sync::atomic::AtomicU32::new(1),
        }
    }
}
