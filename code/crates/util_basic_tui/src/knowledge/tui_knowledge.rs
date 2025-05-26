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

use crate::knowledge::error::LedgeraTuiKnowledgeError;
use ledgera_knowledge_representation::know::LedgeraKnowledgeRepresentation;
use ledgera_node_client::comms::feedback_from_core_client::ValidatedCoreFeedbackMessage;
use ledgera_pki::manager::SerdeSerializable64BitsSignature;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::votes::vout::LedgeraFunctionInstanceOutputKind;
use ledgera_types::{app_template::input::LedgeraInputArgument, digest::LedgeraDigest};
use std::collections::{HashMap, HashSet};

pub struct LedgeraTuiKnowledge<LAT: LedgeraApplicationTemplate> {
    pub cached_client_knowledge: LedgeraKnowledgeRepresentation<LAT>,
    pub data_monikers: HashMap<String, LedgeraDigest>,
    pub computations_monikers: HashMap<String, SerdeSerializable64BitsSignature>,
}

impl<LAT: LedgeraApplicationTemplate> LedgeraTuiKnowledge<LAT> {
    pub fn new() -> Self {
        Self {
            cached_client_knowledge: LedgeraKnowledgeRepresentation::new(),
            data_monikers: HashMap::new(),
            computations_monikers: HashMap::new(),
        }
    }

    pub fn update_on_user_provided_input_raw_value(&mut self, value: LAT::Data, moniker: String) {
        let data_digest = LedgeraDigest::from_serializable(&value).unwrap();
        self.data_monikers.insert(moniker, data_digest.clone());
        self.cached_client_knowledge
            .update_per_application_input_data(data_digest, value);
    }

    pub fn update_on_user_retrieved_raw_value(
        &mut self,
        value_digest: LedgeraDigest,
        value: LAT::Data,
    ) {
        self.cached_client_knowledge
            .update_per_application_input_data(value_digest, value);
    }

    pub fn update_on_user_proposed_function_instance(
        &mut self,
        fid: SerdeSerializable64BitsSignature,
        opt_moniker: Option<String>,
    ) {
        if let Some(moniker) = opt_moniker {
            self.computations_monikers.insert(moniker, fid);
        } else {
            let current_monikers = self.get_all_monikers();
            let mut current_c_index = 1;
            let mut new_moniker = format!("c{}", current_c_index);
            while current_monikers.contains(&new_moniker) {
                current_c_index += 1;
                new_moniker = format!("c{}", current_c_index);
            }
            self.computations_monikers.insert(new_moniker, fid);
        }
    }

    pub fn update_with_core_client_feedback(
        &mut self,
        feedback: ValidatedCoreFeedbackMessage<LAT>,
    ) {
        let (new_data_monikers, new_comp_monikers) =
            self.make_new_monikers_for_new_data_and_comps(&feedback);
        self.cached_client_knowledge
            .update_per_feedback_from_core(feedback);
        self.data_monikers.extend(new_data_monikers);
        self.computations_monikers.extend(new_comp_monikers);
    }

    fn make_new_monikers_for_new_data_and_comps(
        &mut self,
        feedback: &ValidatedCoreFeedbackMessage<LAT>,
    ) -> (
        HashMap<String, LedgeraDigest>,
        HashMap<String, SerdeSerializable64BitsSignature>,
    ) {
        let current_monikers = self.get_all_monikers();
        // ***
        let mut new_data_monikers = HashMap::new();
        let mut current_d_index = 1;
        let data_that_already_have_a_moniker: HashSet<&LedgeraDigest> =
            self.data_monikers.values().collect();
        // ***
        let mut new_comp_monikers = HashMap::new();
        let mut current_c_index = 1;
        let comps_ids_that_already_have_a_moniker: HashSet<&SerdeSerializable64BitsSignature> =
            self.computations_monikers.values().collect();
        // ***
        match feedback {
            ValidatedCoreFeedbackMessage::DeliveredTsto(pos)
                if !data_that_already_have_a_moniker.contains(&pos.v.data_digest) =>
            {
                let mut new_moniker = format!("d{}", current_d_index);
                while current_monikers.contains(&new_moniker) {
                    current_d_index += 1;
                    new_moniker = format!("d{}", current_d_index);
                }
                new_data_monikers.insert(new_moniker, pos.v.data_digest.clone());
            }
            ValidatedCoreFeedbackMessage::ValidatedComputationInstance(
                validated_computation_instance,
            ) => {
                if !comps_ids_that_already_have_a_moniker
                    .contains(&validated_computation_instance.rfun_sig)
                {
                    let mut new_moniker = format!("c{}", current_c_index);
                    while current_monikers.contains(&new_moniker) {
                        current_c_index += 1;
                        new_moniker = format!("c{}", current_c_index);
                    }
                    new_comp_monikers
                        .insert(new_moniker, validated_computation_instance.rfun_sig.clone());
                }
                for input in &validated_computation_instance.rfun.spec.arguments {
                    match input {
                        LedgeraInputArgument::RawValue {
                            is_input_persistent,
                            value,
                        } => {
                            if *is_input_persistent {
                                let value_digest = LedgeraDigest::from_serializable(value).unwrap();
                                if !data_that_already_have_a_moniker.contains(&value_digest) {
                                    let mut new_moniker = format!("d{}", current_d_index);
                                    while current_monikers.contains(&new_moniker) {
                                        current_d_index += 1;
                                        new_moniker = format!("d{}", current_d_index);
                                    }
                                    current_d_index += 1;
                                    new_data_monikers.insert(new_moniker, value_digest);
                                }
                            }
                        }
                        LedgeraInputArgument::ReferenceToStorage(pos) => {
                            if !data_that_already_have_a_moniker.contains(&pos.v.data_digest) {
                                let mut new_moniker = format!("d{}", current_d_index);
                                while current_monikers.contains(&new_moniker) {
                                    current_d_index += 1;
                                    new_moniker = format!("d{}", current_d_index);
                                }
                                current_d_index += 1;
                                new_data_monikers.insert(new_moniker, pos.v.data_digest.clone());
                            }
                        }
                        LedgeraInputArgument::Unknown(_) => {
                            // do nothing
                        }
                    }
                }
            }
            ValidatedCoreFeedbackMessage::Nout(nres) => {
                if let LedgeraFunctionInstanceOutputKind::ComputedOutput {
                    is_output_persistent: _,
                    output_digest,
                } = &nres.poi.v.result_kind
                {
                    if !data_that_already_have_a_moniker.contains(output_digest) {
                        let mut new_moniker = format!("d{}", current_d_index);
                        while current_monikers.contains(&new_moniker) {
                            current_d_index += 1;
                            new_moniker = format!("d{}", current_d_index);
                        }
                        new_data_monikers.insert(new_moniker, output_digest.clone());
                    }
                }
            }
            _ => {}
        }
        // ***
        (new_data_monikers, new_comp_monikers)
    }

    pub fn get_all_monikers(&self) -> HashSet<&String> {
        let mut key_set: HashSet<&String> = HashSet::new();
        key_set.extend(self.data_monikers.keys());
        key_set.extend(self.computations_monikers.keys());
        key_set
    }

    pub fn rename_moniker(
        &mut self,
        original: &String,
        new: String,
    ) -> Result<bool, LedgeraTuiKnowledgeError> {
        if *original == new {
            return Ok(false);
        }

        if self.get_all_monikers().contains(&new) {
            return Err(LedgeraTuiKnowledgeError::AlreadyUsedMoniker);
        }

        if self.data_monikers.contains_key(original) {
            let val = self.data_monikers.remove(original).unwrap();
            self.data_monikers.insert(new, val);
            return Ok(true);
        }
        if self.computations_monikers.contains_key(original) {
            let val = self.computations_monikers.remove(original).unwrap();
            self.computations_monikers.insert(new, val);
            return Ok(true);
        }

        Err(LedgeraTuiKnowledgeError::UnknownMoniker)
    }
}

impl<LAT: LedgeraApplicationTemplate> Default for LedgeraTuiKnowledge<LAT> {
    fn default() -> Self {
        Self::new()
    }
}
