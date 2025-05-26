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

use ledgera_pki::manager::KnownParticipantsMap;
use ledgera_pki::quorum::QuorumOfSignatures;

use crate::{error::LedgeraInternalApiError, votes::vfun::LedgeraVoteFunctionInstanceDeclaration};

const PROOF_OF_FUNCTION_DECLARATION_STR: &str = "ProofOfFunctionDeclaration";

#[derive(Debug, PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProofOfFunctionDeclaration {
    // the vote below contains necessary information to distinguish unambiguously the operation instance
    pub v: LedgeraVoteFunctionInstanceDeclaration,
    // a quorum of signature on the above vote
    pub q: QuorumOfSignatures,
}

impl ProofOfFunctionDeclaration {
    pub fn new(v: LedgeraVoteFunctionInstanceDeclaration, q: QuorumOfSignatures) -> Self {
        Self { v, q }
    }

    pub fn verify_proof_of_operation_declaration<
        PKI: ledgera_pki::manager::PublicKeyInfrastructure,
    >(
        &self,
        known_participants: &KnownParticipantsMap<PKI::VerifyingKey>,
        threshold: u32,
    ) -> Result<(), LedgeraInternalApiError> {
        super::verify_quorum_agreement::<PKI, _>(
            &self.v,
            &self.q,
            known_participants,
            threshold,
            PROOF_OF_FUNCTION_DECLARATION_STR,
        )
    }
}
