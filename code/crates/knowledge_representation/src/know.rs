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
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::digest::LedgeraDigest;
use std::collections::HashMap;

use crate::{
    per_data::LedgeraDataValueKnowledgeRepresentation,
    per_function_instance::LedgeraFunctionInstanceKnowledgeRepresentation,
};

pub struct LedgeraKnowledgeRepresentation<LAT: LedgeraApplicationTemplate> {
    /// keeps track of info relative to all the data values
    pub per_data_value: HashMap<LedgeraDigest, LedgeraDataValueKnowledgeRepresentation<LAT>>,
    /// keeps track of info relative to all the function instances
    pub per_function_instance: HashMap<
        SerdeSerializable64BitsSignature,
        LedgeraFunctionInstanceKnowledgeRepresentation<LAT>,
    >,
    pub function_instances_order_in_log: Vec<SerdeSerializable64BitsSignature>,
}

impl<LAT: LedgeraApplicationTemplate> Clone for LedgeraKnowledgeRepresentation<LAT> {
    fn clone(&self) -> Self {
        Self {
            per_data_value: self.per_data_value.clone(),
            per_function_instance: self.per_function_instance.clone(),
            function_instances_order_in_log: self.function_instances_order_in_log.clone(),
        }
    }
}

impl<LAT: LedgeraApplicationTemplate> PartialEq for LedgeraKnowledgeRepresentation<LAT> {
    fn eq(&self, other: &Self) -> bool {
        self.per_data_value == other.per_data_value
            && self.per_function_instance == other.per_function_instance
            && self.function_instances_order_in_log == other.function_instances_order_in_log
    }
}

impl<LAT: LedgeraApplicationTemplate> Eq for LedgeraKnowledgeRepresentation<LAT> {}
impl<LAT: LedgeraApplicationTemplate> Default for LedgeraKnowledgeRepresentation<LAT> {
    fn default() -> Self {
        Self {
            per_data_value: HashMap::new(),
            per_function_instance: HashMap::new(),
            function_instances_order_in_log: Vec::new(),
        }
    }
}

impl<LAT: LedgeraApplicationTemplate> LedgeraKnowledgeRepresentation<LAT> {
    pub fn new() -> Self {
        Self {
            per_data_value: HashMap::new(),
            per_function_instance: HashMap::new(),
            function_instances_order_in_log: Vec::new(),
        }
    }
}
