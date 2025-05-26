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

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

use crate::error::LedgeraPkiError;
use crate::manager::{PublicKeyInfrastructure, SerdeSerializable64BitsSignature};

pub struct DefaultPublicKeyInfrastructureBackend {}

impl PublicKeyInfrastructure for DefaultPublicKeyInfrastructureBackend {
    type SigningKey = SigningKey;
    type VerifyingKey = VerifyingKey;
    type Signature = Signature;

    fn generate_signing_key() -> Self::SigningKey {
        let mut csprng = OsRng {};
        SigningKey::generate(&mut csprng)
    }

    fn get_verifying_key_from_signing_key(signing_key: &Self::SigningKey) -> Self::VerifyingKey {
        signing_key.verifying_key()
    }

    fn sign_message(signing_key: &Self::SigningKey, message: &[u8]) -> Self::Signature {
        signing_key.sign(message)
    }

    fn verify_signature(
        verifying_key: &Self::VerifyingKey,
        message: &[u8],
        signature: &Self::Signature,
    ) -> Result<(), LedgeraPkiError> {
        match verifying_key.verify(message, signature) {
            Ok(_) => Ok(()),
            Err(_) => Err(LedgeraPkiError::SignatureFailedVerification),
        }
    }

    fn serialize_verifying_key(verifying_key: &VerifyingKey) -> [u8; 32] {
        verifying_key.to_bytes()
    }

    /// https://docs.rs/ed25519-dalek/latest/src/ed25519_dalek/verifying.rs.html#130
    fn deserialize_as_verifying_key(
        bytes: &[u8; 32],
    ) -> Result<Self::VerifyingKey, LedgeraPkiError> {
        match Self::VerifyingKey::from_bytes(bytes) {
            Ok(vk) => Ok(vk),
            Err(_) => Err(LedgeraPkiError::CannotDeserializeVerifyingKey),
        }
    }

    fn from_clear_signature_to_serializable_signature(
        clear_signature: &Self::Signature,
    ) -> SerdeSerializable64BitsSignature {
        let signature_bytes = clear_signature.to_bytes();

        // Split the signature into two 32-byte parts
        let part1: [u8; 32] = signature_bytes[0..32].try_into().unwrap();
        let part2: [u8; 32] = signature_bytes[32..64].try_into().unwrap();

        SerdeSerializable64BitsSignature::new(part1, part2)
    }

    fn from_serializable_signature_to_clear_signature(
        serializable_signature: &SerdeSerializable64BitsSignature,
    ) -> Self::Signature {
        // Combine the two 32-byte parts of the signature into a single 64-byte array
        let mut signature_bytes = [0u8; 64];
        signature_bytes[0..32].copy_from_slice(serializable_signature.get_part1());
        signature_bytes[32..64].copy_from_slice(serializable_signature.get_part2());

        // Deserialize the signature from the combined bytes
        Signature::from(&signature_bytes)
    }
}
