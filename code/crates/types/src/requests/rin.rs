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

use std::collections::BTreeSet;

use ledgera_pki::manager::SerdeSerializable64BitsSignature;

use crate::error::{LedgeraInternalApiError, LedgeraInternalApiErrorContext};
use crate::proofs::proof_of_storage::ProofOfShipmentToStorage;
use crate::traits::{LedgeraPublishableMessage, LedgeraQuorumContainingMessage};

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct LedgeraRequestInputProposal {
    // unambiguous identification of the instance of the computation (via the signature of the initial message that declared it)
    pub function_instance_identifier: SerdeSerializable64BitsSignature,
    // the argument proposal may be used as the argument at any of the indicies in "argument_indices"
    pub argument_indices: BTreeSet<u32>,
    // the actual value must correspond to a reference to storage to ensure data availability on all honest/correct Ledgera voters
    pub input_data: ProofOfShipmentToStorage,
}

impl LedgeraRequestInputProposal {
    pub fn new(
        function_instance_identifier: SerdeSerializable64BitsSignature,
        argument_indices: BTreeSet<u32>,
        input_data: ProofOfShipmentToStorage,
    ) -> Self {
        Self {
            function_instance_identifier,
            argument_indices,
            input_data,
        }
    }
}

impl LedgeraPublishableMessage for LedgeraRequestInputProposal {
    fn get_msg_type() -> &'static str {
        "Rin"
    }
}

impl LedgeraQuorumContainingMessage for LedgeraRequestInputProposal {
    fn verify_vote_quorums<PKI: ledgera_pki::manager::PublicKeyInfrastructure>(
        &self,
        known_participants: &ledgera_pki::manager::KnownParticipantsMap<PKI::VerifyingKey>,
        threshold: u32,
    ) -> Result<(), crate::error::LedgeraInternalApiError> {
        match self
            .input_data
            .verify_proof_of_shipment_to_storage::<PKI>(known_participants, threshold)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(LedgeraInternalApiError::InContext(
                LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                Box::new(e),
            )),
        }
    }
}
