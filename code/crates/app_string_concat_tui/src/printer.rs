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

use ledgera_knowledge_representation::printer::LedgeraComputationItemsPrinter;

use ledgera_app_string_concat::lat_binding::*;
pub struct StrConcatComputationPrinter {}

impl LedgeraComputationItemsPrinter<StrConcatBackend> for StrConcatComputationPrinter {
    fn print_tag(tag: &StrConcatTag) -> String {
        match tag {
            StrConcatTag::Tag => "tag".to_string(),
        }
    }

    fn print_computation(cmp: &StrConcatComputation) -> String {
        match cmp {
            StrConcatComputation::Concat => "concat".to_string(),
        }
    }

    fn print_value(v: &StrConcatData) -> String {
        v.string.clone()
    }

    fn print_local_predicate(p: &StrConcatLocalPredicate) -> String {
        match p {
            StrConcatLocalPredicate::StringLongerThan(x) => {
                format!("(>{})", x)
            }
        }
    }

    fn print_global_predicate(p: &StrConcatGlobalPredicate) -> String {
        match p {
            StrConcatGlobalPredicate::PairwiseDistinct => "distinct".to_string(),
        }
    }
}
