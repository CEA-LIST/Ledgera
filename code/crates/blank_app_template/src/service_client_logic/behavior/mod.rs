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
use ledgera_node_client::runtime::runtime_io::CoreClientRuntime;
use ledgera_pki::manager::{PublicKeyInfrastructure, SerdeSerializable64BitsSignature};

use ledgera_types::app_template::input::LedgeraInputArgument;
use ledgera_types::app_template::operation::LedgeraAtomicOperation;
use ledgera_types::app_template::spec::LedgeraAtomicOperationSpecification;

use crate::lat_binding::{
    LedgeraServiceTemplate, LedgeraServiceTemplateData,
    LedgeraServiceTemplateMultiArgumentsPredicate, LedgeraServiceTemplateOperation,
    LedgeraServiceTemplateSingularArgumentPredicate, LedgeraServiceTemplateTag,
};

pub mod react_to_core;
pub mod react_to_peer;
pub mod react_to_user;

pub struct LedgeraServiceClientBehavior<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork> {
    pub(super) comm_session:
        Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    pub(super) comm_params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
    pub(super) service: Arc<LedgeraServiceTemplate>,
    pub(super) core_client_runtime_io: CoreClientRuntime<PKI, Sess, LedgeraServiceTemplate>,
    // TODO : add other attributes to represent internal state
}

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork> LedgeraServiceClientBehavior<PKI, Sess> {
    pub fn new(
        comm_session: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
        comm_params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
        service: Arc<LedgeraServiceTemplate>,
        core_client_runtime_io: CoreClientRuntime<PKI, Sess, LedgeraServiceTemplate>,
    ) -> Self {
        Self {
            comm_session,
            comm_params,
            service,
            core_client_runtime_io,
        }
    }

    // Submits an atomic operation to Core and logs the outcome.
    // Single helper shared by all branches in `react_to_user`.
    #[allow(dead_code)]
    pub(super) async fn submit_and_log(
        &self,
        operation: LedgeraAtomicOperation<
            LedgeraServiceTemplateTag,
            LedgeraServiceTemplateOperation,
        >,
        arguments: Vec<
            LedgeraInputArgument<
                LedgeraServiceTemplateData,
                LedgeraServiceTemplateSingularArgumentPredicate,
            >,
        >,
        log_label: &str,
    ) -> Option<SerdeSerializable64BitsSignature> {
        let spec = LedgeraAtomicOperationSpecification::new(
            operation,
            None::<LedgeraServiceTemplateMultiArgumentsPredicate>,
            arguments,
        );
        let res = self
            .core_client_runtime_io
            .compute_function(spec)
            .await
            .ok();
        log::info!("{}", log_label);
        let _ = (&self.comm_session, &self.comm_params, &self.service);
        res
    }
}
