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

use std::collections::{BTreeMap, BTreeSet};

use ledgera_pki::manager::{KnownParticipantsMap, SerdeSerializable64BitsSignature};
use ledgera_pki::message::SignatureEntry;

use crate::proofs::proof_of_storage::ProofOfShipmentToStorage;
use crate::requests::rin::LedgeraRequestInputProposal;
use crate::traits::LedgeraPublishableMessage;

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize, Hash)]
pub struct LedgeraVoteInsInputProposalReference {
    // signature of the original "Rin" request to which this object constitutes a reference
    pub signature_of_rin: SignatureEntry,
    // "argument_indices" attribute of the "Rin" to be able to verify the "signature_of_rin" signature
    pub argument_indices: BTreeSet<u32>,
    // proof of storage for the value that is proposed:
    // - used to check the input proposal against the local and global predicate
    //   whenever a voter receives a "Vins" vote and has to check it and echo it to reach a quorum
    // - also used as the "input_data" attribute of the reconstituted "Rin" to be able to verify the "signature_of_rin" signature
    pub pos: ProofOfShipmentToStorage,
}

impl LedgeraVoteInsInputProposalReference {
    pub fn new(
        signature_of_rin: SignatureEntry,
        argument_indices: BTreeSet<u32>,
        pos: ProofOfShipmentToStorage,
    ) -> Self {
        Self {
            signature_of_rin,
            argument_indices,
            pos,
        }
    }

    pub(crate) fn verify_it_corresponds_to_a_real_rin<
        PKI: ledgera_pki::manager::PublicKeyInfrastructure,
    >(
        &self,
        function_instance_identifier: &SerdeSerializable64BitsSignature,
        known_participants: &KnownParticipantsMap<PKI::VerifyingKey>,
        threshold: u32,
    ) -> Result<(), LedgeraVoteInsMalformationError> {
        let reconstituted_rin = LedgeraRequestInputProposal::new(
            function_instance_identifier.clone(),
            self.argument_indices.clone(),
            self.pos.clone(),
        );
        let signature = PKI::from_serializable_signature_to_clear_signature(
            &self.signature_of_rin.serializable_signature,
        );
        let owner_public_key =
            PKI::deserialize_as_verifying_key(&self.signature_of_rin.serialized_signing_public_key)
                .map_err(|_| LedgeraVoteInsMalformationError::InvalidSigningPublicKey)?;
        PKI::verify_signature(
            &owner_public_key,
            &bincode::serialize(&reconstituted_rin).unwrap(),
            &signature,
        )
        .map_err(|_| LedgeraVoteInsMalformationError::SignatureVerificationFailed)?;
        // Def. 11: lps must be a valid LP_S (quorum of f+1 signatures on the Vsto vote).
        self.pos
            .verify_proof_of_shipment_to_storage::<PKI>(known_participants, threshold)
            .map_err(|_| LedgeraVoteInsMalformationError::InvalidLpsQuorum)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize, Hash)]
pub struct LedgeraVoteIns {
    /// identifies the operation instance via the signature of the initial 'Rfun' message that requested its declaration and execution
    pub function_instance_identifier: SerdeSerializable64BitsSignature,
    /// proposed assignment of unknow variables to "Rin" requests that fulfills them
    pub proposed_unknowns_assignment: BTreeMap<u32, LedgeraVoteInsInputProposalReference>,
}

#[derive(Debug, Clone)]
pub enum LedgeraVoteInsMalformationError {
    InvalidSigningPublicKey,
    SignatureVerificationFailed,
    ArgumentIndexMismatch,
    InvalidLpsQuorum,
}

impl LedgeraVoteIns {
    pub fn new(
        function_instance_identifier: SerdeSerializable64BitsSignature,
        proposed_unknowns_assignment: BTreeMap<u32, LedgeraVoteInsInputProposalReference>,
    ) -> Self {
        Self {
            function_instance_identifier,
            proposed_unknowns_assignment,
        }
    }

    /**
     * The "Vins" contains an assigment that, to each unknwon input of the function instance,
     * associates a reference to an input proposal.
     * With this function, we verify that all these references correspond to real "Rin" messages.
     * **/
    pub fn verify_traceability_of_each_input_to_real_rin<
        PKI: ledgera_pki::manager::PublicKeyInfrastructure,
    >(
        &self,
        known_participants: &KnownParticipantsMap<PKI::VerifyingKey>,
        threshold: u32,
    ) -> Result<(), LedgeraVoteInsMalformationError> {
        for (arg_idx, vins_arg_ref) in &self.proposed_unknowns_assignment {
            vins_arg_ref.verify_it_corresponds_to_a_real_rin::<PKI>(
                &self.function_instance_identifier,
                known_participants,
                threshold,
            )?;
            if !vins_arg_ref.argument_indices.contains(arg_idx) {
                return Err(LedgeraVoteInsMalformationError::ArgumentIndexMismatch);
            }
        }
        Ok(())
    }
}

impl LedgeraPublishableMessage for LedgeraVoteIns {
    fn get_msg_type() -> &'static str {
        "Vins"
    }
}
