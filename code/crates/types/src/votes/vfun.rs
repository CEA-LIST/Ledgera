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

use std::collections::{BTreeMap, BTreeSet};

use ledgera_pki::manager::SerdeSerializable64BitsSignature;

use crate::digest::LedgeraDigest;
use crate::traits::LedgeraPublishableMessage;

/**
 * Vfun vote emitted by voters to confirm execute access for a computation instance
 * This vote contains all required information so that
 * secure log nodes and storage nodes may reason about
 * the computation instance without having access to its full specification
 * (they only have a quorum of such Vfun votes).
 *
 * This includes:
 * - information about unknown arguments (so that secure log nodes might know if a "Tins" is needed)
 * - wether or not the operation is a computation or a simple tag (so that secure log nodes might know if a Tout is needed)
 * - the indices of the raw inputs that are persistent (so that storage nodes might only accept valid storage requests)
 * **/
#[derive(Debug, PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize)]
pub struct LedgeraVoteFunctionInstanceDeclaration {
    // the signature of the Rfun message that declared the operation instance
    pub function_instance_identifier: SerdeSerializable64BitsSignature,
    // the digests of the concrete values of all known positional arguments, indexed by their positions
    pub known_arguments: BTreeMap<u32, LedgeraDigest>,
    // the indices of the positional arguments that are not provided values (which will need to later be filled-in via 'Rin' requests)
    pub unknown_arguments_indices: BTreeSet<u32>,
    // indices of the raw inputs that are persistent (and thus ought to be stored)
    pub persistent_inputs_indices: BTreeSet<u32>,
}

impl LedgeraVoteFunctionInstanceDeclaration {
    pub fn new(
        function_instance_identifier: SerdeSerializable64BitsSignature,
        known_arguments: BTreeMap<u32, LedgeraDigest>,
        unknown_arguments_indices: BTreeSet<u32>,
        persistent_inputs_indices: BTreeSet<u32>,
    ) -> Self {
        Self {
            function_instance_identifier,
            known_arguments,
            unknown_arguments_indices,
            persistent_inputs_indices,
        }
    }
}

impl LedgeraPublishableMessage for LedgeraVoteFunctionInstanceDeclaration {
    fn get_msg_type() -> &'static str {
        "Vfun"
    }
}
