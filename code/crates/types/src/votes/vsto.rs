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

use ledgera_pki::manager::SerdeSerializable64BitsSignature;

use crate::traits::LedgeraPublishableMessage;

use crate::digest::LedgeraDigest;

#[derive(
    Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize, Hash, PartialOrd, Ord,
)]
pub enum PersistentDataKind {
    // if the persistent data we want to store is a positional input of the operation
    Input(u32),
    // if the persistent data we want to store is an output of the operation
    // TODO: if later-on we have a list of outputs instead of a single one
    Output,
}

impl std::fmt::Display for PersistentDataKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistentDataKind::Input(x) => {
                write!(f, "input@{:}", x)
            }
            PersistentDataKind::Output => {
                write!(f, "output")
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize, Hash)]
/// to build promises of storage, "voter" nodes exchange votes
pub struct LedgeraVoteStored {
    // identifier of the instance of persistent atomic operation that led to the storage request
    pub function_instance_identifier: SerdeSerializable64BitsSignature,
    // digest of the data that is stored
    pub data_digest: LedgeraDigest,
    // kind of the data
    pub data_kind: PersistentDataKind,
}

impl LedgeraVoteStored {
    pub fn new(
        function_instance_identifier: SerdeSerializable64BitsSignature,
        data_digest: LedgeraDigest,
        data_kind: PersistentDataKind,
    ) -> Self {
        Self {
            function_instance_identifier,
            data_digest,
            data_kind,
        }
    }
}

impl LedgeraPublishableMessage for LedgeraVoteStored {
    fn get_msg_type() -> &'static str {
        "Vsto"
    }
}
