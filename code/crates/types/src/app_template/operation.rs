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

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub enum LedgeraAtomicOperation<Tag, Computation> {
    TagInputs(Tag),
    ComputeOutput {
        is_output_persistent: bool,
        comp: Computation,
    },
}

impl<Tag, Computation> LedgeraAtomicOperation<Tag, Computation> {
    pub fn is_tag_inputs(&self) -> bool {
        matches!(self, LedgeraAtomicOperation::TagInputs(_))
    }
    pub fn is_output_persistent(&self) -> bool {
        match self {
            LedgeraAtomicOperation::TagInputs(_) => false,
            LedgeraAtomicOperation::ComputeOutput {
                is_output_persistent,
                ..
            } => *is_output_persistent,
        }
    }
}

/**
 * Specifies an operation that is atomic and that produces an output from a list of inputs.
 *  **/
pub trait LedgeraAtomicComputation<DataValue, RuntimeError> {
    fn compute(
        &self,
        arguments: Vec<DataValue>,
    ) -> impl std::future::Future<Output = Result<DataValue, RuntimeError>> + Send;
}

/**
 * Specifies an operation that is atomic and that does not produce an output.
 *  **/
pub trait LedgeraAtomicTag {}
