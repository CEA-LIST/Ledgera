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

use crate::error::LedgeraInternalApiError;
use crate::votes::vsto::LedgeraVoteStored;
use ledgera_pki::quorum::QuorumOfSignatures;

const PROOF_OF_SHIPMENT_TO_STORAGE_STR: &str = "ProofOfShipmentToStorage";

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize, Hash)]
pub struct ProofOfShipmentToStorage {
    pub v: LedgeraVoteStored,
    pub q: QuorumOfSignatures,
}

impl ProofOfShipmentToStorage {
    pub fn new(v: LedgeraVoteStored, q: QuorumOfSignatures) -> Self {
        Self { v, q }
    }

    pub fn verify_proof_of_shipment_to_storage<PKI: PublicKeyInfrastructure>(
        &self,
        known_participants: &KnownParticipantsMap<PKI::VerifyingKey>,
        threshold: u32,
    ) -> Result<(), LedgeraInternalApiError> {
        super::verify_quorum_agreement::<PKI, _>(
            &self.v,
            &self.q,
            known_participants,
            threshold,
            PROOF_OF_SHIPMENT_TO_STORAGE_STR,
        )
    }
}
