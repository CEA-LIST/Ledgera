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
use ledgera_core_logic::{queries::query_data::query_data_from_storage, roles::LedgeraCoreRoles};
use ledgera_pki::manager::PublicKeyInfrastructure;
use ledgera_types::{
    app_template::template::LedgeraApplicationTemplate, digest::LedgeraDigest,
    proofs::proof_of_storage::ProofOfShipmentToStorage,
};

use crate::runtime::runtime_io::CoreClientRuntime;

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork, LAT: LedgeraApplicationTemplate>
    CoreClientRuntime<PKI, Sess, LAT>
{
    pub async fn retrieve_data(
        &self,
        data_digest: LedgeraDigest,
        pos: Option<ProofOfShipmentToStorage>,
    ) -> Result<Option<LAT::Data>, tokio::sync::oneshot::error::RecvError> {
        log::info!(
            "As {:?} : processing LedgeraRequestToCoreClient::RetrieveDataValueFromStorage",
            LedgeraCoreRoles::Client
        );
        let (value_sender, value_receiver) = tokio::sync::oneshot::channel();
        let _ = query_data_from_storage::<PKI, Sess, LAT>(
            &self.service_ref,
            self.comm_session_ref.clone(),
            self.comm_params_ref.clone(),
            data_digest,
            pos,
            value_sender,
        )
        .await;
        match value_receiver.await {
            Ok(got_resp) => Ok(got_resp),
            Err(e) => Err(e),
        }
    }
}
