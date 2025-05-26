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

use crate::app_template::identifier::ComputationInstanceInternalIdentifier;
use crate::app_template::input::LedgeraInputArgument;
use crate::app_template::spec::LedgeraAtomicOperationSpecification;
use crate::app_template::template::LedgeraApplicationTemplate;
use crate::error::{LedgeraInternalApiError, LedgeraInternalApiErrorContext};
use crate::traits::{LedgeraPublishableMessage, LedgeraQuorumContainingMessage};

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound(
    serialize = "
        LAT::Data: serde::Serialize,
        LAT::Tag: serde::Serialize,
        LAT::Computation: serde::Serialize,
        LAT::LocalPredicate: serde::Serialize,
        LAT::GlobalPredicate: serde::Serialize,
    ",
    deserialize = "
        LAT::Data: serde::Deserialize<'de>,
        LAT::Tag: serde::Deserialize<'de>,
        LAT::Computation: serde::Deserialize<'de>,
        LAT::LocalPredicate: serde::Deserialize<'de>,
        LAT::GlobalPredicate: serde::Deserialize<'de>,
    "
))]
pub struct LedgeraRequestFunctionInstanceProposal<LAT: LedgeraApplicationTemplate> {
    pub id: ComputationInstanceInternalIdentifier,
    pub spec: LedgeraAtomicOperationSpecification<LAT>,
}

impl<LAT: LedgeraApplicationTemplate> LedgeraRequestFunctionInstanceProposal<LAT> {
    pub fn new(
        id: ComputationInstanceInternalIdentifier,
        spec: LedgeraAtomicOperationSpecification<LAT>,
    ) -> Self {
        Self { id, spec }
    }
}

impl<LAT: LedgeraApplicationTemplate> LedgeraPublishableMessage
    for LedgeraRequestFunctionInstanceProposal<LAT>
{
    fn get_msg_type() -> &'static str {
        "Rfun"
    }
}

impl<LAT: LedgeraApplicationTemplate> LedgeraQuorumContainingMessage
    for LedgeraRequestFunctionInstanceProposal<LAT>
{
    fn verify_vote_quorums<PKI: ledgera_pki::manager::PublicKeyInfrastructure>(
        &self,
        known_participants: &ledgera_pki::manager::KnownParticipantsMap<PKI::VerifyingKey>,
        threshold: u32,
    ) -> Result<(), crate::error::LedgeraInternalApiError> {
        for arg in &self.spec.arguments {
            if let LedgeraInputArgument::ReferenceToStorage(reference) = arg {
                if let Err(e) = reference
                    .verify_proof_of_shipment_to_storage::<PKI>(known_participants, threshold)
                {
                    return Err(LedgeraInternalApiError::InContext(
                        LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                        Box::new(e),
                    ));
                }
            }
        }
        Ok(())
    }
}
