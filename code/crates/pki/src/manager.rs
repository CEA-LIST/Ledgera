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

use std::collections::HashMap;

use crate::error::LedgeraPkiError;

pub const PKI_SERIALIZED_PUBLIC_KEY_LENGTH: usize = 32;

/// A map from serialized public-key bytes to the deserialized verifying key.
/// Used as the canonical representation of the known-participants set throughout the codebase,
/// enabling O(1) membership checks keyed directly on the raw bytes carried in SignatureEntry.
pub type KnownParticipantsMap<VK> = HashMap<[u8; PKI_SERIALIZED_PUBLIC_KEY_LENGTH], VK>;

/// serde cannot serialize [u8;64] automatically
/// so we split it like that
/// For the signature, we need it to implement serde::Serialize, serde::Deserialize
/// so that we may use bincode::serialize(signature)
#[derive(
    Debug, PartialEq, Eq, Clone, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Hash,
)]
pub struct SerdeSerializable64BitsSignature {
    part1: [u8; 32],
    part2: [u8; 32],
}

impl SerdeSerializable64BitsSignature {
    pub fn new(part1: [u8; 32], part2: [u8; 32]) -> Self {
        Self { part1, part2 }
    }
    pub fn get_part1(&self) -> &[u8] {
        &self.part1
    }
    pub fn get_part2(&self) -> &[u8] {
        &self.part2
    }
    pub fn to_hexadecimal_string(&self) -> String {
        format!("{}{}", hex::encode(self.part1), hex::encode(self.part2))
    }
}

pub trait PublicKeyInfrastructure: Send + Sync + 'static {
    type SigningKey: PartialEq + Eq + Clone + std::fmt::Debug + Send + Sync + 'static;

    type VerifyingKey: PartialEq + Eq + Clone + std::fmt::Debug + Send + Sync + 'static;

    type Signature: PartialEq + Eq + Clone + std::fmt::Debug;

    fn generate_signing_key() -> Self::SigningKey;

    fn get_verifying_key_from_signing_key(signing_key: &Self::SigningKey) -> Self::VerifyingKey;

    fn sign_message(signing_key: &Self::SigningKey, serialized_payload: &[u8]) -> Self::Signature;

    fn verify_signature(
        verifying_key: &Self::VerifyingKey,
        serialized_payload: &[u8],
        signature: &Self::Signature,
    ) -> Result<(), LedgeraPkiError>;

    fn serialize_verifying_key(
        verifying_key: &Self::VerifyingKey,
    ) -> [u8; PKI_SERIALIZED_PUBLIC_KEY_LENGTH];

    fn deserialize_as_verifying_key(
        bytes: &[u8; PKI_SERIALIZED_PUBLIC_KEY_LENGTH],
    ) -> Result<Self::VerifyingKey, LedgeraPkiError>;

    fn from_clear_signature_to_serializable_signature(
        clear_signature: &Self::Signature,
    ) -> SerdeSerializable64BitsSignature;

    fn from_serializable_signature_to_clear_signature(
        serializable_signature: &SerdeSerializable64BitsSignature,
    ) -> Self::Signature;
}
