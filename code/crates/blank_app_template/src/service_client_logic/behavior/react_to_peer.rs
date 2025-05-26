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

use ledgera_comms::comm_session::PubSubNetwork;
use ledgera_pki::manager::PublicKeyInfrastructure;
use ledgera_pki::message::SignatureEntry;

use crate::service_client_logic::LedgeraServiceTemplateType1Message;

use super::LedgeraServiceClientBehavior;

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork> LedgeraServiceClientBehavior<PKI, Sess> {
    pub async fn react_to_service_type1_msg(
        &mut self,
        _service_msg: LedgeraServiceTemplateType1Message,
        _msg_sig: SignatureEntry,
    ) {
        // TODO : how, upon receiving a service message, does the Service client :
        // - updates its internal state
        // - react by sending other messages either directly or via sending a request to the co-located Ledgera Core client
    }
}
