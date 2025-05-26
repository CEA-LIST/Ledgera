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
use ledgera_pki::manager::{PublicKeyInfrastructure, SerdeSerializable64BitsSignature};
use ledgera_types::{
    app_template::{
        identifier::ComputationInstanceInternalIdentifier,
        spec::LedgeraAtomicOperationSpecification, template::LedgeraApplicationTemplate,
    },
    requests::rfun::LedgeraRequestFunctionInstanceProposal,
};

use crate::runtime::runtime_io::CoreClientRuntime;

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork, LAT: LedgeraApplicationTemplate>
    CoreClientRuntime<PKI, Sess, LAT>
{
    pub async fn compute_function(
        &self,
        function_specification: LedgeraAtomicOperationSpecification<LAT>,
        // TODO: handle of synchronous function calls
        // is_synchronous : bool
    ) -> Result<SerdeSerializable64BitsSignature, ()> {
        log::info!(
            "As {:?} : processing LedgeraRequestToCoreClient::SubmitComputationProposal",
            LedgeraCoreRoles::Client
        );
        let rfun_request = {
            let client_specific_comp_id = self
                .next_computation_id_when_submitting
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let comp_id = ComputationInstanceInternalIdentifier::new(
                PKI::serialize_verifying_key(&PKI::get_verifying_key_from_signing_key(
                    &self.comm_params_ref.signing_key,
                )),
                client_specific_comp_id,
            );
            LedgeraRequestFunctionInstanceProposal::new(comp_id, function_specification)
        };
        {
            let mut comm_sess = self.comm_session_ref.lock().await;
            match comm_sess
                .serialize_and_publish_on_topic_returning_signature::<LedgeraRequestFunctionInstanceProposal<LAT>>(
                    &self.comm_params_ref,
                    &LedgeraCorePublicationTopics::Rfun.get_publication_topic_str(self.service_ref.as_ref()),
                    &rfun_request
                ).await {
                Err(e) => {
                    log::warn!(
                        "As {:?} : could not emit client-side computation proposal with error : {:?}",
                        LedgeraCoreRoles::Client,
                        e
                    );
                    Err(())
                },
                Ok(function_instance_id) => {
                    log::info!(
                        "As {:?} : sent a LedgeraRequestFunctionInstanceProposal following user request",
                        LedgeraCoreRoles::Client
                    );
                    Ok(function_instance_id)
                }
            }
        }
    }
}
