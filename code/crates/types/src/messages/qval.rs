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

use crate::{
    digest::LedgeraDigest,
    error::{LedgeraInternalApiError, LedgeraInternalApiErrorContext},
    proofs::proof_of_storage::ProofOfShipmentToStorage,
    traits::{LedgeraPublishableMessage, LedgeraQuorumContainingMessage},
};

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct LedgeraQueryValue {
    pub digest_of_value: LedgeraDigest,
    pub pos_opt: Option<ProofOfShipmentToStorage>,
}

impl LedgeraQueryValue {
    pub fn new(digest_of_value: LedgeraDigest, pos_opt: Option<ProofOfShipmentToStorage>) -> Self {
        Self {
            digest_of_value,
            pos_opt,
        }
    }
}

impl LedgeraPublishableMessage for LedgeraQueryValue {
    fn get_msg_type() -> &'static str {
        "Qval"
    }
}

impl LedgeraQuorumContainingMessage for LedgeraQueryValue {
    fn verify_vote_quorums<PKI: ledgera_pki::manager::PublicKeyInfrastructure>(
        &self,
        known_participants: &ledgera_pki::manager::KnownParticipantsMap<PKI::VerifyingKey>,
        threshold: u32,
    ) -> Result<(), crate::error::LedgeraInternalApiError> {
        if let Some(pos) = &self.pos_opt {
            if pos.v.data_digest != self.digest_of_value {
                return Err(LedgeraInternalApiError::InContext(
                    LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                    Box::new(LedgeraInternalApiError::QuorumAgreedUponValueDoesNotMatchContext),
                ));
            }
            if let Err(e) =
                pos.verify_proof_of_shipment_to_storage::<PKI>(known_participants, threshold)
            {
                return Err(LedgeraInternalApiError::InContext(
                    LedgeraInternalApiErrorContext::WhenVerifying(Self::get_msg_type()),
                    Box::new(e),
                ));
            }
        }
        Ok(())
    }
}
