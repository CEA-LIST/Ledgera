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

use crate::service_client_logic::role::LEDGERA_VARKEEP_SERVICE_CLIENT_ROLE;
use crate::service_client_logic::service_msgs::messages::LedgeraVarkeepServicePublishLocVarMsg;

use super::LedgeraServiceClientBehavior;

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork> LedgeraServiceClientBehavior<PKI, Sess> {
    pub async fn react_to_service_msg_publish_local_var(
        &mut self,
        publish_msg: LedgeraVarkeepServicePublishLocVarMsg,
        sig: SignatureEntry,
    ) {
        let client_id = if let Some(position) = self
            .clients
            .iter()
            .position(|x| x == &sig.serialized_signing_public_key)
        {
            position
        } else {
            let position = self.clients.len();
            self.clients.push(sig.serialized_signing_public_key);
            position
        };
        let clientname = format!("client{:}", client_id);
        log::info!(
            "As {:} : receive update of local variable {:} of client {:}",
            LEDGERA_VARKEEP_SERVICE_CLIENT_ROLE,
            publish_msg.varname,
            clientname
        );
        let vn = format!("{:}.{:}", clientname, publish_msg.varname);
        let _ = self.to_ui_feed.send((vn, publish_msg.varvalue)).await;
    }
}
