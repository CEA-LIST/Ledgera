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

use std::collections::{HashMap, HashSet};

use ledgera_pki::manager::SerdeSerializable64BitsSignature;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::{
    digest::LedgeraDigest, proofs::proof_of_storage::ProofOfShipmentToStorage,
    votes::vsto::PersistentDataKind,
};

/// everything that the client currently knows about a specific computation instance
pub struct LedgeraDataValueKnowledgeRepresentation<LAT: LedgeraApplicationTemplate> {
    pub data_value: Option<LAT::Data>,
    pub data_digest: LedgeraDigest,

    pub proofs_of_storage: HashMap<
        (SerdeSerializable64BitsSignature, PersistentDataKind),
        HashSet<ProofOfShipmentToStorage>,
    >,
}

impl<LAT: LedgeraApplicationTemplate> Clone for LedgeraDataValueKnowledgeRepresentation<LAT> {
    fn clone(&self) -> Self {
        Self {
            data_value: self.data_value.clone(),
            data_digest: self.data_digest.clone(),
            proofs_of_storage: self.proofs_of_storage.clone(),
        }
    }
}

impl<LAT: LedgeraApplicationTemplate> PartialEq for LedgeraDataValueKnowledgeRepresentation<LAT> {
    fn eq(&self, other: &Self) -> bool {
        self.data_value == other.data_value
            && self.data_digest == other.data_digest
            && self.proofs_of_storage == other.proofs_of_storage
    }
}

impl<LAT: LedgeraApplicationTemplate> Eq for LedgeraDataValueKnowledgeRepresentation<LAT> {}

impl<LAT: LedgeraApplicationTemplate> LedgeraDataValueKnowledgeRepresentation<LAT> {
    pub fn new(data_digest: LedgeraDigest) -> Self {
        Self {
            data_value: None,
            data_digest,
            proofs_of_storage: HashMap::new(),
        }
    }

    pub fn process_pos(&mut self, pos: ProofOfShipmentToStorage) {
        let key = (
            pos.v.function_instance_identifier.clone(),
            pos.v.data_kind.clone(),
        );
        let poss = self.proofs_of_storage.entry(key).or_default();
        poss.insert(pos);
    }

    pub fn process_value(&mut self, value: LAT::Data) {
        self.data_value = Some(value);
    }

    pub fn garbage_collect_value(&mut self) {
        self.data_value = None;
    }
}
