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

use ledgera_macros::{
    ledgera_application_template, ledgera_computation, ledgera_data, ledgera_error,
    ledgera_global_predicate, ledgera_local_predicate, ledgera_tag,
};
use ledgera_pki::manager::SerdeSerializable64BitsSignature;

#[ledgera_data]
pub enum LedgeraServiceTemplateData {
    // TODO : fill with the various types of data
    // that might be used by your service
    Placeholder,
}

#[ledgera_error]
pub enum LedgeraServiceTemplateRuntimeError {
    DefaultError,
    // TODO : fill with errors that might be
    // returned at runtime by your service
}

#[ledgera_computation]
pub enum LedgeraServiceTemplateOperation {
    // TODO : fill with the various types of operations of your service
    // that you want to be done as Ledgera Core computation instances
    Placeholder,
}

impl LedgeraServiceTemplateOperation {
    pub async fn ledgera_compute(
        &self,
        _arguments: Vec<LedgeraServiceTemplateData>,
    ) -> Result<LedgeraServiceTemplateData, LedgeraServiceTemplateRuntimeError> {
        Err(LedgeraServiceTemplateRuntimeError::DefaultError)
    }
}

#[ledgera_tag]
pub enum LedgeraServiceTemplateTag {
    // TODO : add variants here for each kind of `TagInputs` operation your service needs.
    // A `TagInputs(tag)` operation stores inputs without producing an output,
    // so that a later `ComputeOutput` can reference them. Leave empty if all your operations
    // produce an output directly (see `LedgeraAtomicOperation`).
    Placeholder,
}

#[ledgera_local_predicate]
pub enum LedgeraServiceTemplateSingularArgumentPredicate {
    // TODO : fill with the predicates that you may define
    // to constraint unknown arguments of Ledgera Core computation instances
    Placeholder,
}

impl LedgeraServiceTemplateSingularArgumentPredicate {
    pub fn ledgera_single_arg_is_valid(
        &self,
        _value: &LedgeraServiceTemplateData,
        _function_instance_identifier: &SerdeSerializable64BitsSignature,
    ) -> Result<bool, LedgeraServiceTemplateRuntimeError> {
        Err(LedgeraServiceTemplateRuntimeError::DefaultError)
    }
}

#[ledgera_global_predicate]
pub struct LedgeraServiceTemplateMultiArgumentsPredicate;

impl LedgeraServiceTemplateMultiArgumentsPredicate {
    pub fn ledgera_multi_args_is_valid(
        &self,
        _arguments: &[&LedgeraServiceTemplateData],
    ) -> Result<bool, LedgeraServiceTemplateRuntimeError> {
        Err(LedgeraServiceTemplateRuntimeError::DefaultError)
    }
}

#[ledgera_application_template(
    name                      = "template",
    data                      = LedgeraServiceTemplateData,
    computation               = LedgeraServiceTemplateOperation,
    tag                       = LedgeraServiceTemplateTag,
    local_predicate = LedgeraServiceTemplateSingularArgumentPredicate,
    global_predicate = LedgeraServiceTemplateMultiArgumentsPredicate,
    error                     = LedgeraServiceTemplateRuntimeError,
)]
#[derive(Debug, Clone)]
pub struct LedgeraServiceTemplate;
