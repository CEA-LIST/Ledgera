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

use crate::error::LedgeraPkiError;
use crate::manager::{
    KnownParticipantsMap, PublicKeyInfrastructure, PKI_SERIALIZED_PUBLIC_KEY_LENGTH,
};
use crate::message::SignatureEntry;
use std::collections::{BTreeSet, HashSet};

#[derive(
    Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize, Ord, PartialOrd, Hash,
)]
pub struct QuorumOfSignatures {
    pub agreed_upon_value: Vec<u8>,
    pub signatures: BTreeSet<SignatureEntry>,
}

impl QuorumOfSignatures {
    pub fn new(agreed_upon_value: Vec<u8>, signatures: BTreeSet<SignatureEntry>) -> Self {
        Self {
            agreed_upon_value,
            signatures,
        }
    }

    pub fn is_a_valid_quorum<PKI: PublicKeyInfrastructure>(
        &self,
        known_participants: &KnownParticipantsMap<PKI::VerifyingKey>,
        threshold: u32,
    ) -> Result<(), LedgeraPkiError> {
        let mut counted_keys: HashSet<[u8; PKI_SERIALIZED_PUBLIC_KEY_LENGTH]> = HashSet::new();
        for sig_entry in &self.signatures {
            if counted_keys.contains(&sig_entry.serialized_signing_public_key) {
                continue;
            }
            if let Some(verifying_key) =
                known_participants.get(&sig_entry.serialized_signing_public_key)
            {
                let clear_signature = PKI::from_serializable_signature_to_clear_signature(
                    &sig_entry.serializable_signature,
                );
                if PKI::verify_signature(verifying_key, &self.agreed_upon_value, &clear_signature)
                    .is_ok()
                {
                    counted_keys.insert(sig_entry.serialized_signing_public_key);
                    if counted_keys.len() > threshold as usize {
                        return Ok(());
                    }
                }
            }
        }
        Err(LedgeraPkiError::QuorumDoesNotMeetThreshold)
    }
}
