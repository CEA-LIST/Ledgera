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

use ledgera_macros::*;
use ledgera_pki::manager::SerdeSerializable64BitsSignature;

#[ledgera_data]
pub enum VarkeepData {
    VariableName(String),
    VariableValue(String),
}

#[ledgera_error]
pub enum VarkeepRuntimeError {
    DefaultError,
}

#[ledgera_computation]
pub enum VarkeepComputation {
    Placeholder,
}

impl VarkeepComputation {
    pub async fn ledgera_compute(
        &self,
        _arguments: Vec<VarkeepData>,
    ) -> Result<VarkeepData, VarkeepRuntimeError> {
        Err(VarkeepRuntimeError::DefaultError)
    }
}

#[ledgera_tag]
pub enum VarkeepTag {
    Assign,
}

#[ledgera_local_predicate]
pub enum VarkeepLocalPredicate {
    IsVarName,
    IsVarValue,
}

impl VarkeepLocalPredicate {
    pub fn ledgera_single_arg_is_valid(
        &self,
        value: &VarkeepData,
        _function_instance_identifier: &SerdeSerializable64BitsSignature,
    ) -> Result<bool, VarkeepRuntimeError> {
        let x = match self {
            Self::IsVarName => {
                matches!(value, VarkeepData::VariableName(_))
            }
            Self::IsVarValue => {
                matches!(value, VarkeepData::VariableValue(_))
            }
        };
        Ok(x)
    }
}

#[ledgera_global_predicate]
pub struct VarkeepGlobalPredicate;

impl VarkeepGlobalPredicate {
    pub fn ledgera_multi_args_is_valid(
        &self,
        _arguments: &[&VarkeepData],
    ) -> Result<bool, VarkeepRuntimeError> {
        Ok(true)
    }
}

#[ledgera_application_template(
    name             = "varkeep",
    data             = VarkeepData,
    computation      = VarkeepComputation,
    tag              = VarkeepTag,
    local_predicate  = VarkeepLocalPredicate,
    global_predicate = VarkeepGlobalPredicate,
    error            = VarkeepRuntimeError,
)]
#[derive(Debug, Clone)]
pub struct LedgeraVarkeepService;
