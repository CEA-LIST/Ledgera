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

use ledgera_pki::manager::{
    PublicKeyInfrastructure, SerdeSerializable64BitsSignature, PKI_SERIALIZED_PUBLIC_KEY_LENGTH,
};

pub struct LedgeraDataOwnershipIdentifier {
    pub public_key: [u8; PKI_SERIALIZED_PUBLIC_KEY_LENGTH],
}

pub enum LedgeraDataAccessPolicy {
    /// correct/honest storage nodes will always accept reads  
    Public,
    /// correct/honest storage nodes will only accept reads that provide a valid credential
    Private,
}

/// for private data, Ledgera voters may access them
/// within the context of a given computation instance
/// only if they have a "LedgeraStorageReadCredential"
pub struct LedgeraStorageReadCredential {
    pub function_instance_identifier: SerdeSerializable64BitsSignature,
    pub signature_by_owner: SerdeSerializable64BitsSignature,
}

impl LedgeraStorageReadCredential {
    pub fn verify_against_data_owner<PKI: PublicKeyInfrastructure>(
        &self,
        owner_id: &LedgeraDataOwnershipIdentifier,
    ) -> bool {
        match PKI::deserialize_as_verifying_key(&owner_id.public_key) {
            Ok(owner_public_key) => {
                let sig =
                    PKI::from_serializable_signature_to_clear_signature(&self.signature_by_owner);
                PKI::verify_signature(
                    &owner_public_key,
                    self.function_instance_identifier
                        .to_hexadecimal_string()
                        .as_bytes(),
                    &sig,
                )
                .is_ok()
            }
            Err(_) => false,
        }
    }

    pub fn create_credential_for_computation_instance<PKI: PublicKeyInfrastructure>(
        private_key: &PKI::SigningKey,
        function_instance_identifier: SerdeSerializable64BitsSignature,
    ) -> Self {
        let signature_by_owner = PKI::sign_message(
            private_key,
            function_instance_identifier
                .to_hexadecimal_string()
                .as_bytes(),
        );
        let signature_by_owner =
            PKI::from_clear_signature_to_serializable_signature(&signature_by_owner);
        Self {
            function_instance_identifier,
            signature_by_owner,
        }
    }
}
