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

use ledgera_pki::manager::{KnownParticipantsMap, PublicKeyInfrastructure};

use crate::{
    digest::LedgeraDigest,
    error::{
        LedgeraInternalApiError, LedgeraInternalApiErrorContext, ServerSideStorageRequestError,
    },
    proofs::{
        proof_of_declaration::ProofOfFunctionDeclaration,
        proof_of_integrity::ProofOfOperationIntegrity,
    },
    traits::{
        LedgeraCommunicatableItem, LedgeraPublishableMessage, LedgeraQuorumContainingMessage,
    },
    votes::{vout::LedgeraFunctionInstanceOutputKind, vsto::PersistentDataKind},
};

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct LedgeraServerSideStorageRequest<DataValue> {
    // value that we want to store on the distributed storage
    pub data_value: DataValue,
    // under what pretence we want to store this value (either as an output or as an input of a function instance)
    pub data_kind: PersistentDataKind,
    // proof that the corresponding function instance has been declared and validated
    // in case the value to store is the "i^{th} input" of that function instance,
    // we need to verify in "function_instance_proof_of_declaration" that:
    // - the "i^{th}" input is indeed flagged as a persistent input
    // - the digest of the "i^{th}" input indeed corresponds to that of "data_value"
    pub function_instance_proof_of_declaration: ProofOfFunctionDeclaration,
    // in case the value is stored as an output, we need a proof that this value indeed corresponds to the output of the function instance execution
    pub opt_function_instance_proof_of_integrity: Option<ProofOfOperationIntegrity>,
}

impl<DataValue> LedgeraServerSideStorageRequest<DataValue> {
    pub fn new(
        data_value: DataValue,
        data_kind: PersistentDataKind,
        function_instance_proof_of_declaration: ProofOfFunctionDeclaration,
        opt_function_instance_proof_of_integrity: Option<ProofOfOperationIntegrity>,
    ) -> Self {
        Self {
            data_value,
            data_kind,
            function_instance_proof_of_declaration,
            opt_function_instance_proof_of_integrity,
        }
    }
}

impl<DataValue: LedgeraCommunicatableItem> LedgeraPublishableMessage
    for LedgeraServerSideStorageRequest<DataValue>
{
    fn get_msg_type() -> &'static str {
        "Rsto"
    }
}

impl<DataValue: LedgeraCommunicatableItem> LedgeraQuorumContainingMessage
    for LedgeraServerSideStorageRequest<DataValue>
{
    fn verify_vote_quorums<PKI: ledgera_pki::manager::PublicKeyInfrastructure>(
        &self,
        known_participants: &ledgera_pki::manager::KnownParticipantsMap<PKI::VerifyingKey>,
        threshold: u32,
    ) -> Result<(), LedgeraInternalApiError> {
        match &self.data_kind {
            PersistentDataKind::Input(input_at_index) => {
                // ***
                if !self
                    .function_instance_proof_of_declaration
                    .v
                    .persistent_inputs_indices
                    .contains(input_at_index)
                {
                    return Err(LedgeraInternalApiError::InContext(
                        LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                        Box::new(LedgeraInternalApiError::Storage(
                            ServerSideStorageRequestError::OnlyRawInputsThatAreTaggedPersistentInQuorumedVfunAreAllowedToBeStored)
                        ),
                    ));
                }
                // ***
                let digest_of_value_to_store =
                    get_digest_of_value_to_store_and_verify_declaration::<DataValue, PKI>(
                        known_participants,
                        threshold,
                        &self.data_value,
                        &self.function_instance_proof_of_declaration,
                    )
                    .map_err(|e| {
                        LedgeraInternalApiError::InContext(
                            LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                            Box::new(e),
                        )
                    })?;
                // ***
                if self
                    .function_instance_proof_of_declaration
                    .v
                    .known_arguments
                    .contains_key(input_at_index)
                {
                    // if the input is already declared in the operation specification
                    // we only need to verify that the 'digest_of_value_to_store' matches that in the Vfun
                    let expected_value_digest = self
                        .function_instance_proof_of_declaration
                        .v
                        .known_arguments
                        .get(input_at_index)
                        .unwrap();
                    if digest_of_value_to_store != *expected_value_digest {
                        return Err(LedgeraInternalApiError::InContext(
                            LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                            Box::new(LedgeraInternalApiError::Storage(
                                ServerSideStorageRequestError::PersistentRawInputDigestDoNotMatchExpectedDigestInQuorumedVfun
                            )
                            ),
                        ));
                    } else {
                        return Ok(());
                    }
                } else if self
                    .function_instance_proof_of_declaration
                    .v
                    .unknown_arguments_indices
                    .contains(input_at_index)
                {
                    // if the input is an unknown, it must be in any case filled-in using an existing Proof Of Storage
                    // so we do not authorize it to be flagged as a persistent input
                    return Err(LedgeraInternalApiError::InContext(
                        LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                        Box::new(LedgeraInternalApiError::Storage(
                            ServerSideStorageRequestError::PersistenceOfUnknownInputsNotAuthorized,
                        )),
                    ));
                } else {
                    return Err(LedgeraInternalApiError::InContext(
                        LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                        Box::new(LedgeraInternalApiError::InputArgumentPositionIsNotDeclared),
                    ));
                }
            }
            PersistentDataKind::Output => match &self.opt_function_instance_proof_of_integrity {
                None => {
                    return Err(LedgeraInternalApiError::InContext(
                            LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                            Box::new(LedgeraInternalApiError::Storage(
                                ServerSideStorageRequestError::CannotStorePersistentOutputWithoutAProofOfIntegrity)
                            ),
                        ));
                }
                Some(poi) => {
                    if let Err(e) = poi
                        .verify_proof_of_operation_integrity::<PKI>(known_participants, threshold)
                    {
                        return Err(LedgeraInternalApiError::InContext(
                            LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                            Box::new(e),
                        ));
                    }

                    if poi.v.function_instance_identifier
                        != self
                            .function_instance_proof_of_declaration
                            .v
                            .function_instance_identifier
                    {
                        return Err(LedgeraInternalApiError::InContext(
                            LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                            Box::new(
                                LedgeraInternalApiError::MismatchInOperationInstanceIdentifiers,
                            ),
                        ));
                    }

                    let digest_of_value_to_store =
                        get_digest_of_value_to_store_and_verify_declaration::<DataValue, PKI>(
                            known_participants,
                            threshold,
                            &self.data_value,
                            &self.function_instance_proof_of_declaration,
                        )
                        .map_err(|e| {
                            LedgeraInternalApiError::InContext(
                                LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                                Box::new(e),
                            )
                        })?;

                    match &poi.v.result_kind {
                        LedgeraFunctionInstanceOutputKind::TaggedInputs => {
                            return Err(LedgeraInternalApiError::InContext(
                                    LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                                    Box::new(LedgeraInternalApiError::Storage(
                                        ServerSideStorageRequestError::TryingToStoreOutputInATagInputsOperation)
                                    ),
                                ));
                        }
                        LedgeraFunctionInstanceOutputKind::ComputedOutput {
                            is_output_persistent,
                            output_digest: poi_output_digest,
                        } => {
                            if !is_output_persistent {
                                return Err(LedgeraInternalApiError::InContext(
                                        LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                                        Box::new(LedgeraInternalApiError::Storage(
                                            ServerSideStorageRequestError::OnlyAnOutputThatIsTaggedPersistentInQuorumedVoutIsAllowedToBeStored)
                                        ),
                                    ));
                            }

                            if digest_of_value_to_store != *poi_output_digest {
                                return Err(LedgeraInternalApiError::InContext(
                                        LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                                        Box::new(LedgeraInternalApiError::Storage(
                                            ServerSideStorageRequestError::PersistentOutputDigestDoNotMatchExpectedDigestInQuorumedVout)
                                        ),
                                    ));
                            }
                        }
                    }
                }
            },
        }
        Ok(())
    }
}

fn get_digest_of_value_to_store_and_verify_declaration<
    DataValue: LedgeraCommunicatableItem,
    PKI: PublicKeyInfrastructure,
>(
    known_participants: &KnownParticipantsMap<PKI::VerifyingKey>,
    threshold: u32,
    data_value: &DataValue,
    operation_instance_declaration: &ProofOfFunctionDeclaration,
) -> Result<LedgeraDigest, LedgeraInternalApiError> {
    match LedgeraDigest::from_serializable(data_value) {
        Ok(digest_of_value_to_store) => {
            match operation_instance_declaration
                .verify_proof_of_operation_declaration::<PKI>(known_participants, threshold)
            {
                Err(e) => Err(e),
                Ok(_) => Ok(digest_of_value_to_store),
            }
        }
        Err(e) => Err(e),
    }
}
