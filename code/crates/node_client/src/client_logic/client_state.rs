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
use ledgera_core_logic::roles::LedgeraCoreRoles;
use ledgera_pki::manager::PublicKeyInfrastructure;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use std::sync::Arc;

pub struct LedgeraClientNodeState<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
> {
    pub(crate) comm_session:
        Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    pub(crate) comm_params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
    pub(crate) service: Arc<LAT>,
    //
    pub(crate) handle_internal_messages_task: Option<tokio::task::JoinHandle<()>>,
    //
    pub(crate) handle_user_requests_task: Option<tokio::task::JoinHandle<()>>,
}

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork, LAT: LedgeraApplicationTemplate>
    LedgeraClientNodeState<PKI, Sess, LAT>
{
    pub fn new(
        comm_session: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
        comm_params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
        service: Arc<LAT>,
    ) -> Self {
        Self {
            comm_session,
            comm_params,
            service,
            handle_internal_messages_task: None,
            handle_user_requests_task: None,
        }
    }

    pub async fn abort(&mut self) {
        if let Some(x) = &self.handle_internal_messages_task {
            log::warn!(
                "As {:?} : aborting handle_internal_messages_task",
                LedgeraCoreRoles::Client
            );
            x.abort();
        }
        if let Some(x) = &self.handle_user_requests_task {
            log::warn!(
                "As {:?} : aborting handle_user_requests_task",
                LedgeraCoreRoles::Client
            );
            x.abort();
        }
    }
}
