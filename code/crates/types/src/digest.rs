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

use crate::error::LedgeraInternalApiError;
use sha3::{Digest, Keccak256};

#[derive(
    Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize, Hash, PartialOrd, Ord,
)]
pub struct LedgeraDigest {
    bytes_of_hash: [u8; 32],
}

impl LedgeraDigest {
    pub fn to_hexadecimal_string(&self) -> String {
        hex::encode(self.bytes_of_hash)
    }

    pub fn from_hexadecimal_string(string: &str) -> Option<Self> {
        match hex::decode(string) {
            Ok(arr) => match <[u8; 32]>::try_from(arr) {
                Ok(bytes_of_hash) => Some(Self { bytes_of_hash }),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }
}

impl LedgeraDigest {
    pub fn serialize_digest(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn from_serializable<T: serde::Serialize>(
        value: &T,
    ) -> Result<Self, LedgeraInternalApiError> {
        match bincode::serialize(value) {
            Ok(bytes) => Ok(Self {
                bytes_of_hash: Keccak256::digest(bytes).into(),
            }),
            Err(_) => Err(LedgeraInternalApiError::CannotProduceDigestOfData),
        }
    }
}
