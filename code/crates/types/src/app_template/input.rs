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

use std::fmt::Debug;

use crate::proofs::proof_of_storage::ProofOfShipmentToStorage;

/**
 A (potentially) abstract positional argument in a ledgera computation specification.
- if the argument is already provided as a concrete value, we have a "LedgeraInputData"
- if the argument is not yet known but must satisfy some constraints, we have a
 **/
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub enum LedgeraInputArgument<DataValue, Predicate> {
    /// a raw value
    RawValue {
        is_input_persistent: bool,
        value: DataValue,
    },
    /// a reference to a value stored on the storage
    ReferenceToStorage(ProofOfShipmentToStorage),
    /// a value that is not known in advance but must uphold a certain predicate
    Unknown(Predicate),
}

impl<DataValue, Predicate> LedgeraInputArgument<DataValue, Predicate> {
    pub fn is_concrete(&self) -> bool {
        !matches!(self, Self::Unknown(_))
    }
    pub fn get_predicate(&self) -> Option<&Predicate> {
        match self {
            LedgeraInputArgument::Unknown(pred) => Some(pred),
            _ => None,
        }
    }
}
