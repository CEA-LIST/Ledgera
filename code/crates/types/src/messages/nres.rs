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

use crate::digest::LedgeraDigest;
use crate::error::{LedgeraInternalApiError, LedgeraInternalApiErrorContext};
use crate::proofs::proof_of_integrity::ProofOfOperationIntegrity;
use crate::traits::{
    LedgeraCommunicatableItem, LedgeraPublishableMessage, LedgeraQuorumContainingMessage,
};
use crate::votes::vout::LedgeraFunctionInstanceOutputKind;

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct LedgeraComputationResultNotification<DataValue> {
    pub result_value: DataValue,
    pub poi: ProofOfOperationIntegrity,
}

impl<DataValue> LedgeraComputationResultNotification<DataValue> {
    pub fn new(result_value: DataValue, poi: ProofOfOperationIntegrity) -> Self {
        Self { result_value, poi }
    }
}

impl<DataValue: LedgeraCommunicatableItem> LedgeraPublishableMessage
    for LedgeraComputationResultNotification<DataValue>
{
    fn get_msg_type() -> &'static str {
        "Nres"
    }
}

impl<DataValue: LedgeraCommunicatableItem> LedgeraQuorumContainingMessage
    for LedgeraComputationResultNotification<DataValue>
{
    fn verify_vote_quorums<PKI: ledgera_pki::manager::PublicKeyInfrastructure>(
        &self,
        known_participants: &ledgera_pki::manager::KnownParticipantsMap<PKI::VerifyingKey>,
        threshold: u32,
    ) -> Result<(), crate::error::LedgeraInternalApiError> {
        let res_digest = LedgeraDigest::from_serializable(&self.result_value).unwrap();
        match &self.poi.v.result_kind {
            LedgeraFunctionInstanceOutputKind::TaggedInputs => {
                // there should not be a 'nres' in case of a tag inputs operation
                return Err(LedgeraInternalApiError::InContext(
                    LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                    Box::new(LedgeraInternalApiError::ANresShouldNotExistForATagInputsOperation),
                ));
            }
            LedgeraFunctionInstanceOutputKind::ComputedOutput {
                is_output_persistent: _,
                output_digest: poi_output_digest,
            } => {
                if res_digest != *poi_output_digest {
                    return Err(LedgeraInternalApiError::InContext(
                        LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                        Box::new(LedgeraInternalApiError::QuorumAgreedUponValueDoesNotMatchContext),
                    ));
                }
            }
        };
        match self
            .poi
            .verify_proof_of_operation_integrity::<PKI>(known_participants, threshold)
        {
            Ok(()) => Ok(()),
            Err(e) => Err(LedgeraInternalApiError::InContext(
                LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                Box::new(e),
            )),
        }
    }
}
