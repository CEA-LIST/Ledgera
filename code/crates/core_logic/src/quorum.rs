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

use std::collections::{BTreeSet, HashSet};

use ledgera_pki::{
    manager::{PublicKeyInfrastructure, PKI_SERIALIZED_PUBLIC_KEY_LENGTH},
    message::SignatureEntry,
    quorum::QuorumOfSignatures,
};
use log::warn;

pub async fn collect_quorum<PKI: PublicKeyInfrastructure>(
    value: Vec<u8>,
    mut vote_receiver: tokio::sync::mpsc::Receiver<SignatureEntry>,
    quorum_threshold: usize,
) -> Option<QuorumOfSignatures> {
    let mut q = BTreeSet::new();
    let mut counted_keys: HashSet<[u8; PKI_SERIALIZED_PUBLIC_KEY_LENGTH]> = HashSet::new();
    while let Some(sig_entry) = vote_receiver.recv().await {
        if counted_keys.contains(&sig_entry.serialized_signing_public_key) {
            continue;
        }
        let Ok(verifying_key) =
            PKI::deserialize_as_verifying_key(&sig_entry.serialized_signing_public_key)
        else {
            warn!("received signature entry with invalid public key bytes when building quorum");
            continue;
        };
        let is_valid_signature = PKI::verify_signature(
            &verifying_key,
            &value,
            &PKI::from_serializable_signature_to_clear_signature(&sig_entry.serializable_signature),
        );
        if is_valid_signature.is_ok() {
            counted_keys.insert(sig_entry.serialized_signing_public_key);
            q.insert(sig_entry);
            if q.len() > quorum_threshold {
                return Some(QuorumOfSignatures::new(value, q));
            }
        } else {
            warn!("received incorrect signature when building quorum");
        }
    }
    None
}
