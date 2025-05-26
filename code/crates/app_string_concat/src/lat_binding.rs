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
pub struct StrConcatData {
    pub string: String,
}

impl StrConcatData {
    pub fn new(string: String) -> Self {
        Self { string }
    }
}

unsafe impl Send for StrConcatData {}

#[ledgera_computation]
pub enum StrConcatComputation {
    Concat,
}

impl StrConcatComputation {
    pub async fn ledgera_compute(
        &self,
        arguments: Vec<StrConcatData>,
    ) -> Result<StrConcatData, StrConcatRuntimeError> {
        match self {
            StrConcatComputation::Concat => {
                if arguments.len() != 2 {
                    Err(StrConcatRuntimeError::AtomicConcatenationComputationRequiresExactlyTwoStrings)
                } else {
                    let v1 = (*arguments.first().unwrap()).clone();
                    let v2 = (*arguments.get(1).unwrap()).clone();
                    Ok(StrConcatData {
                        string: format!("{}{}", v1.string, v2.string),
                    })
                }
            }
        }
    }
}

#[ledgera_tag]
pub enum StrConcatTag {
    Tag,
}

#[ledgera_local_predicate]
pub enum StrConcatLocalPredicate {
    StringLongerThan(u32),
}

unsafe impl Send for StrConcatLocalPredicate {}

impl StrConcatLocalPredicate {
    pub fn ledgera_single_arg_is_valid(
        &self,
        value: &StrConcatData,
        _function_instance_identifier: &SerdeSerializable64BitsSignature,
    ) -> Result<bool, StrConcatRuntimeError> {
        match self {
            StrConcatLocalPredicate::StringLongerThan(min_length) => {
                if (value.string.len() as u32) < *min_length {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
        }
    }
}

#[ledgera_global_predicate]
pub enum StrConcatGlobalPredicate {
    PairwiseDistinct,
}

unsafe impl Send for StrConcatGlobalPredicate {}

impl StrConcatGlobalPredicate {
    pub fn ledgera_multi_args_is_valid(
        &self,
        arguments: &[&StrConcatData],
    ) -> Result<bool, StrConcatRuntimeError> {
        if arguments.len() != 2 {
            Err(StrConcatRuntimeError::PairwiseDistinctConstraintEvaluationRequiresExactlyTwoStrings)
        } else {
            let s1 = arguments.first().unwrap();
            let s2 = arguments.get(1).unwrap();
            match self {
                StrConcatGlobalPredicate::PairwiseDistinct => Ok(s1 != s2),
            }
        }
    }
}

#[ledgera_error]
pub enum StrConcatRuntimeError {
    AtomicConcatenationComputationRequiresExactlyTwoStrings,
    PairwiseDistinctConstraintEvaluationRequiresExactlyTwoStrings,
    PairwiseDistinctConstraintViolation,
}

unsafe impl Send for StrConcatRuntimeError {}

#[ledgera_application_template(
    name             = "string_concat",
    data             = StrConcatData,
    computation      = StrConcatComputation,
    tag              = StrConcatTag,
    local_predicate  = StrConcatLocalPredicate,
    global_predicate = StrConcatGlobalPredicate,
    error            = StrConcatRuntimeError,
)]
#[derive(Debug)]
pub struct StrConcatBackend {}
