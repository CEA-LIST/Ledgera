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
use ledgera_core_logic::{roles::LedgeraCoreRoles, topics::LedgeraCorePublicationTopics};
use ledgera_pki::manager::PublicKeyInfrastructure;
use ledgera_types::{
    app_template::template::LedgeraApplicationTemplate, requests::rin::LedgeraRequestInputProposal,
};

use crate::runtime::runtime_io::CoreClientRuntime;

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork, LAT: LedgeraApplicationTemplate>
    CoreClientRuntime<PKI, Sess, LAT>
{
    pub async fn propose_input(&self, rin_request: &LedgeraRequestInputProposal) -> Result<(), ()> {
        log::info!(
            "As {:?} : processing LedgeraRequestToCoreClient::SubmitArgumentProposal",
            LedgeraCoreRoles::Client
        );
        let mut comm_sess = self.comm_session_ref.lock().await;
        match comm_sess
            .serialize_and_publish_on_topic::<LedgeraRequestInputProposal>(
                &self.comm_params_ref,
                &LedgeraCorePublicationTopics::Rin
                    .get_publication_topic_str(self.service_ref.as_ref()),
                rin_request,
            )
            .await
        {
            Err(e) => {
                log::warn!(
                    "As {:?} : could not emit client-side storage request with error : {:?}",
                    LedgeraCoreRoles::Client,
                    e
                );
                Err(())
            }
            Ok(()) => {
                log::info!(
                    "As {:?} : sent a LedgeraRequestInputProposal following user request",
                    LedgeraCoreRoles::Client
                );
                Ok(())
            }
        }
    }
}
