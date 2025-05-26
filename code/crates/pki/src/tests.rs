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
use crate::manager::{PublicKeyInfrastructure, PKI_SERIALIZED_PUBLIC_KEY_LENGTH};

fn get_messages() -> Vec<&'static [u8]> {
    vec![
        b"",
        b"message",
        b"Other message",
        b"With, symbols ! @ something",
    ]
}

pub(crate) fn tool_test_message_signing_and_verification<Crypt: PublicKeyInfrastructure>() {
    // Generate keypair (private and public key)
    let signing_key = Crypt::generate_signing_key();
    let verifying_key = Crypt::get_verifying_key_from_signing_key(&signing_key);

    // Test that public key corresponds to the private key by verifying its validity.
    for message in get_messages() {
        let signature = Crypt::sign_message(&signing_key, message);
        let is_valid = Crypt::verify_signature(&verifying_key, message, &signature);
        assert!(is_valid.is_ok());
        for other_message in get_messages() {
            let other_signature = Crypt::sign_message(&signing_key, other_message);
            let is_other_valid =
                Crypt::verify_signature(&verifying_key, other_message, &other_signature);
            if other_message == message {
                assert_eq!(other_signature, signature);
                assert!(is_other_valid.is_ok());
            } else {
                let is_other_valid_wrt_original_signature =
                    Crypt::verify_signature(&verifying_key, other_message, &signature);
                match is_other_valid_wrt_original_signature {
                    Ok(()) => {
                        panic!();
                    }
                    Err(e) => {
                        assert_eq!(e, LedgeraPkiError::SignatureFailedVerification);
                    }
                }
            }
        }
    }
}

pub(crate) fn tool_test_serialize_deserialize_verifying_key<Crypt: PublicKeyInfrastructure>() {
    // Generate keypair (private and public key)
    let signing_key = Crypt::generate_signing_key();
    let verifying_key = Crypt::get_verifying_key_from_signing_key(&signing_key);

    // Serialize the verifying key to bytes
    let serialized_verifying_key = Crypt::serialize_verifying_key(&verifying_key);

    // Deserialize the verifying key from bytes
    let deserialized_verifying_key =
        Crypt::deserialize_as_verifying_key(&serialized_verifying_key).unwrap();

    // Assert that the original verifying key is equal to the deserialized verifying key
    assert_eq!(verifying_key, deserialized_verifying_key);

    for message in get_messages() {
        let signature = Crypt::sign_message(&signing_key, message);
        // Assert that we can verify the signature with both the original verifying key and the deserialized verifying key
        assert!(Crypt::verify_signature(&verifying_key, message, &signature).is_ok());
        assert!(Crypt::verify_signature(&deserialized_verifying_key, message, &signature).is_ok());
    }
}

pub(crate) fn tool_test_serialize_deserialize_signature<Crypt: PublicKeyInfrastructure>() {
    // Generate keypair (private and public key)
    let signing_key = Crypt::generate_signing_key();
    let verifying_key = Crypt::get_verifying_key_from_signing_key(&signing_key);

    for message in get_messages() {
        let clear_signature = Crypt::sign_message(&signing_key, message);
        let serializable_signature =
            Crypt::from_clear_signature_to_serializable_signature(&clear_signature);
        let deserialized_signature =
            Crypt::from_serializable_signature_to_clear_signature(&serializable_signature);
        // Assert that the original signature is equal to the deserialized signature
        assert_eq!(clear_signature, deserialized_signature);
        // Assert that the verifying key can verify both the initial signature and the deserialized signature
        assert!(Crypt::verify_signature(&verifying_key, message, &clear_signature).is_ok());
        assert!(Crypt::verify_signature(&verifying_key, message, &deserialized_signature).is_ok());
    }
}

pub(crate) fn tool_test_deserialize_nonsense_as_verifying_key<Crypt: PublicKeyInfrastructure>() {
    for _ in 0..100 {
        let random_array: [u8; PKI_SERIALIZED_PUBLIC_KEY_LENGTH] = rand::random();
        match Crypt::deserialize_as_verifying_key(&random_array) {
            Ok(vk) => {
                let re_serialized = Crypt::serialize_verifying_key(&vk);
                assert_eq!(random_array, re_serialized);
            }
            Err(e) => {
                assert_eq!(e, LedgeraPkiError::CannotDeserializeVerifyingKey);
            }
        }
    }
}
