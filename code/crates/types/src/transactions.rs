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

use crate::error::{LedgeraInternalApiError, LedgeraInternalApiErrorContext};
use crate::proofs::proof_of_declaration::ProofOfFunctionDeclaration;
use crate::proofs::proof_of_integrity::ProofOfOperationIntegrity;
use crate::proofs::proof_of_storage::ProofOfShipmentToStorage;
use crate::proofs::proof_of_unknown_arguments_assignment_verification::ProofOfUnknownArgumentsAssignmentVerification;
use crate::traits::{LedgeraPublishableMessage, LedgeraQuorumContainingMessage};

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub enum LedgeraTransaction {
    Tsto(ProofOfShipmentToStorage),
    Tfun(ProofOfFunctionDeclaration),
    Tins(ProofOfUnknownArgumentsAssignmentVerification),
    Tout(ProofOfOperationIntegrity),
}

impl LedgeraTransaction {
    pub fn get_transaction_kind(&self) -> &'static str {
        match self {
            LedgeraTransaction::Tsto(_) => "Tsto",
            LedgeraTransaction::Tfun(_) => "Tfun",
            LedgeraTransaction::Tins(_) => "Tins",
            LedgeraTransaction::Tout(_) => "Tout",
        }
    }
}

impl LedgeraPublishableMessage for LedgeraTransaction {
    fn get_msg_type() -> &'static str {
        "transaction"
    }
}

impl LedgeraQuorumContainingMessage for LedgeraTransaction {
    fn verify_vote_quorums<PKI: ledgera_pki::manager::PublicKeyInfrastructure>(
        &self,
        known_participants: &ledgera_pki::manager::KnownParticipantsMap<PKI::VerifyingKey>,
        threshold: u32,
    ) -> Result<(), crate::error::LedgeraInternalApiError> {
        match self {
            LedgeraTransaction::Tsto(pos) => {
                match pos.verify_proof_of_shipment_to_storage::<PKI>(known_participants, threshold)
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(LedgeraInternalApiError::InContext(
                        LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                        Box::new(e),
                    )),
                }
            }
            LedgeraTransaction::Tfun(pod) => {
                match pod
                    .verify_proof_of_operation_declaration::<PKI>(known_participants, threshold)
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(LedgeraInternalApiError::InContext(
                        LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                        Box::new(e),
                    )),
                }
            }
            LedgeraTransaction::Tins(pouav) => {
                match pouav.verify_proof_of_unknown_arguments_assignment_verification::<PKI>(
                    known_participants,
                    threshold,
                ) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(LedgeraInternalApiError::InContext(
                        LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                        Box::new(e),
                    )),
                }
            }
            LedgeraTransaction::Tout(poi) => {
                match poi.verify_proof_of_operation_integrity::<PKI>(known_participants, threshold)
                {
                    Ok(()) => Ok(()),
                    Err(e) => Err(LedgeraInternalApiError::InContext(
                        LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                        Box::new(e),
                    )),
                }
            }
        }
    }
}
