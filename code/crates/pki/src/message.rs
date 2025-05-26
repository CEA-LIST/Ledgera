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
    KnownParticipantsMap, PublicKeyInfrastructure, SerdeSerializable64BitsSignature,
    PKI_SERIALIZED_PUBLIC_KEY_LENGTH,
};

#[derive(
    Debug, PartialEq, Eq, Clone, PartialOrd, Ord, serde::Serialize, serde::Deserialize, Hash,
)]
pub struct SignatureEntry {
    pub serialized_signing_public_key: [u8; PKI_SERIALIZED_PUBLIC_KEY_LENGTH],
    pub serializable_signature: SerdeSerializable64BitsSignature,
}

impl SignatureEntry {
    pub fn new(
        serialized_signing_public_key: [u8; PKI_SERIALIZED_PUBLIC_KEY_LENGTH],
        serializable_signature: SerdeSerializable64BitsSignature,
    ) -> Self {
        Self {
            serialized_signing_public_key,
            serializable_signature,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthenticatableMessage {
    pub serialized_payload: Vec<u8>,
    pub signature_entry: SignatureEntry,
}

impl AuthenticatableMessage {
    pub fn create<T: serde::Serialize, PKI: PublicKeyInfrastructure>(
        payload: &T,
        signing_key: &PKI::SigningKey,
    ) -> Result<Self, LedgeraPkiError> {
        match bincode::serialize(payload) {
            Ok(serialized_payload) => {
                let signature_entry = SignatureEntry::new(
                    PKI::serialize_verifying_key(&PKI::get_verifying_key_from_signing_key(
                        signing_key,
                    )),
                    PKI::from_clear_signature_to_serializable_signature(&PKI::sign_message(
                        signing_key,
                        &serialized_payload[..],
                    )),
                );
                Ok(Self {
                    serialized_payload,
                    signature_entry,
                })
            }
            Err(_) => Err(LedgeraPkiError::CannotSerializeMessagePayload),
        }
    }

    pub fn deserialize_payload<T: for<'a> serde::Deserialize<'a>>(
        &self,
    ) -> Result<T, LedgeraPkiError> {
        match bincode::deserialize(&self.serialized_payload) {
            Ok(msg) => Ok(msg),
            Err(_) => Err(LedgeraPkiError::CannotDeserializeMessagePayload),
        }
    }

    pub fn authenticate<PKI: PublicKeyInfrastructure>(
        &self,
        known_participants: &KnownParticipantsMap<PKI::VerifyingKey>,
    ) -> Result<(), LedgeraPkiError> {
        match known_participants.get(&self.signature_entry.serialized_signing_public_key) {
            None => Err(LedgeraPkiError::UnknownParticipant),
            Some(verifying_key) => {
                let clear_signature = PKI::from_serializable_signature_to_clear_signature(
                    &self.signature_entry.serializable_signature,
                );
                PKI::verify_signature(verifying_key, &self.serialized_payload, &clear_signature)
            }
        }
    }
}
