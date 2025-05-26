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

use derive_where::derive_where;

use crate::app_template::{
    input::LedgeraInputArgument, operation::LedgeraAtomicOperation,
    template::LedgeraApplicationTemplate,
};

/**
The specification of a ledgera function instance (static definition, not dynamic instance being executed).
 **/
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[derive_where(Clone, PartialEq, Eq)]
pub struct LedgeraAtomicOperationSpecification<LAT: LedgeraApplicationTemplate> {
    pub operation: LedgeraAtomicOperation<LAT::Tag, LAT::Computation>,
    pub global_arguments_predicate: Option<LAT::GlobalPredicate>,
    pub arguments: Vec<LedgeraInputArgument<LAT::Data, LAT::LocalPredicate>>,
}

impl<LAT: LedgeraApplicationTemplate> LedgeraAtomicOperationSpecification<LAT> {
    pub fn new(
        operation: LedgeraAtomicOperation<LAT::Tag, LAT::Computation>,
        global_arguments_predicate: Option<LAT::GlobalPredicate>,
        arguments: Vec<LedgeraInputArgument<LAT::Data, LAT::LocalPredicate>>,
    ) -> Self {
        Self {
            operation,
            global_arguments_predicate,
            arguments,
        }
    }
}
