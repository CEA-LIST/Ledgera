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

use std::collections::HashSet;

use ledgera_types::app_template::operation::LedgeraAtomicOperation;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;

use crate::knowledge::error::LedgeraTuiKnowledgeError;
use crate::knowledge::tui_knowledge::LedgeraTuiKnowledge;

#[derive(Debug)]
pub enum LedgeraTuiCommandOperationArgument<LAT: LedgeraApplicationTemplate> {
    Value(LedgeraTuiCommandValueReference<LAT>),
    Predicate(LAT::LocalPredicate),
}

impl<LAT: LedgeraApplicationTemplate> PartialEq for LedgeraTuiCommandOperationArgument<LAT> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                LedgeraTuiCommandOperationArgument::Value(v1),
                LedgeraTuiCommandOperationArgument::Value(v2),
            ) => v1 == v2,
            (
                LedgeraTuiCommandOperationArgument::Predicate(p1),
                LedgeraTuiCommandOperationArgument::Predicate(p2),
            ) => p1 == p2,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum LedgeraTuiCommandValueReference<LAT: LedgeraApplicationTemplate> {
    RawValue {
        is_input_persistent: bool,
        value: LAT::Data,
    },
    ShorthandAsStorageReference(String),
    ShorthandAsRawValue(String),
}

impl<LAT: LedgeraApplicationTemplate> PartialEq for LedgeraTuiCommandValueReference<LAT> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                LedgeraTuiCommandValueReference::RawValue {
                    is_input_persistent: p1,
                    value: v1,
                },
                LedgeraTuiCommandValueReference::RawValue {
                    is_input_persistent: p2,
                    value: v2,
                },
            ) => v1 == v2 && p1 == p2,
            (
                LedgeraTuiCommandValueReference::ShorthandAsStorageReference(s1),
                LedgeraTuiCommandValueReference::ShorthandAsStorageReference(s2),
            ) => s1 == s2,
            (
                LedgeraTuiCommandValueReference::ShorthandAsRawValue(s1),
                LedgeraTuiCommandValueReference::ShorthandAsRawValue(s2),
            ) => s1 == s2,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct LedgeraTuiExecuteCommand<LAT: LedgeraApplicationTemplate> {
    // optionally, we can set a specific moniker for the function instance we want to create
    pub opt_moniker: Option<String>,
    // specifying the function that is executed
    pub operation: LedgeraAtomicOperation<LAT::Tag, LAT::Computation>,
    // specifying the arguments on which the function is applied
    pub arguments: Vec<LedgeraTuiCommandOperationArgument<LAT>>,
    // optionally, we can set a global predicate for the function instance we want to create
    pub opt_global_pred: Option<LAT::GlobalPredicate>,
}

impl<LAT: LedgeraApplicationTemplate> LedgeraTuiExecuteCommand<LAT> {
    pub fn new(
        opt_moniker: Option<String>,
        operation: LedgeraAtomicOperation<LAT::Tag, LAT::Computation>,
        arguments: Vec<LedgeraTuiCommandOperationArgument<LAT>>,
        opt_global_pred: Option<LAT::GlobalPredicate>,
    ) -> Self {
        Self {
            opt_moniker,
            operation,
            arguments,
            opt_global_pred,
        }
    }
}

impl<LAT: LedgeraApplicationTemplate> PartialEq for LedgeraTuiExecuteCommand<LAT> {
    fn eq(&self, other: &Self) -> bool {
        (self.opt_moniker == other.opt_moniker)
            && (self.operation == other.operation)
            && (self.arguments == other.arguments)
    }
}

#[derive(Debug)]
pub enum LedgeraTuiCommand<LAT: LedgeraApplicationTemplate> {
    Exit,
    PrintGraph,
    Rename(String, String),
    GetValue(String),
    Execute(LedgeraTuiExecuteCommand<LAT>),
    PushArg {
        comp_moniker: String,
        arg_potential_indices: HashSet<u32>,
        data_moniker: String,
    },
    AuditValue(LedgeraTuiCommandValueReference<LAT>),
    AuditComputation(String),
}

impl<LAT: LedgeraApplicationTemplate> PartialEq for LedgeraTuiCommand<LAT> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LedgeraTuiCommand::Exit, LedgeraTuiCommand::Exit) => true,
            (LedgeraTuiCommand::PrintGraph, LedgeraTuiCommand::PrintGraph) => true,
            (LedgeraTuiCommand::Rename(n1, n2), LedgeraTuiCommand::Rename(n3, n4)) => {
                (n1 == n3) && (n2 == n4)
            }
            (LedgeraTuiCommand::GetValue(s1), LedgeraTuiCommand::GetValue(s2)) => s1 == s2,
            (LedgeraTuiCommand::Execute(c1), LedgeraTuiCommand::Execute(c2)) => c1 == c2,
            (
                LedgeraTuiCommand::PushArg {
                    comp_moniker: comp_moniker1,
                    arg_potential_indices: arg_potential_indices1,
                    data_moniker: data_moniker1,
                },
                LedgeraTuiCommand::PushArg {
                    comp_moniker: comp_moniker2,
                    arg_potential_indices: arg_potential_indices2,
                    data_moniker: data_moniker2,
                },
            ) => {
                (comp_moniker1 == comp_moniker2)
                    && (arg_potential_indices1 == arg_potential_indices2)
                    && (data_moniker1 == data_moniker2)
            }
            _ => false,
        }
    }
}

impl<LAT: LedgeraApplicationTemplate> LedgeraTuiCommand<LAT> {
    pub fn check_monikers(
        &self,
        k: &LedgeraTuiKnowledge<LAT>,
    ) -> Result<(), LedgeraTuiKnowledgeError> {
        match self {
            LedgeraTuiCommand::AuditValue(audited_value_ref) => {
                match audited_value_ref {
                    LedgeraTuiCommandValueReference::RawValue { .. } => {}
                    LedgeraTuiCommandValueReference::ShorthandAsStorageReference(data_moniker) => {
                        if !k.data_monikers.contains_key(data_moniker) {
                            return Err(LedgeraTuiKnowledgeError::UnknownDataMoniker);
                        }
                    }
                    LedgeraTuiCommandValueReference::ShorthandAsRawValue(data_moniker) => {
                        if k.data_monikers.contains_key(data_moniker) {
                            let data_digest = k.data_monikers.get(data_moniker).unwrap();
                            if let Some(data_k) =
                                k.cached_client_knowledge.per_data_value.get(data_digest)
                            {
                                if data_k.data_value.is_none() {
                                    return Err(LedgeraTuiKnowledgeError::ConcreteValueAtGivenDataMonikerUnknown);
                                }
                            } else {
                                return Err(LedgeraTuiKnowledgeError::ConcreteValueAtGivenDataMonikerUnknown);
                            }
                        } else {
                            return Err(LedgeraTuiKnowledgeError::UnknownDataMoniker);
                        }
                    }
                }
            }
            LedgeraTuiCommand::AuditComputation(comp_moniker)
                if !k.computations_monikers.contains_key(comp_moniker) =>
            {
                return Err(LedgeraTuiKnowledgeError::UnknownComputationMoniker);
            }
            LedgeraTuiCommand::Rename(original, new) => {
                let all_monikers = k.get_all_monikers();
                if !all_monikers.contains(original) {
                    return Err(LedgeraTuiKnowledgeError::UnknownMoniker);
                }
                if all_monikers.contains(new) {
                    return Err(LedgeraTuiKnowledgeError::AlreadyUsedMoniker);
                }
            }
            LedgeraTuiCommand::GetValue(data_moniker)
                if !k.data_monikers.contains_key(data_moniker) =>
            {
                return Err(LedgeraTuiKnowledgeError::UnknownDataMoniker);
            }
            LedgeraTuiCommand::Execute(exec_command) => {
                let all_monikers = k.get_all_monikers();
                if let Some(mon) = &exec_command.opt_moniker {
                    if all_monikers.contains(mon) {
                        return Err(LedgeraTuiKnowledgeError::AlreadyUsedMoniker);
                    }
                }
                for arg in &exec_command.arguments {
                    match arg {
                        LedgeraTuiCommandOperationArgument::Value(vref) => match vref {
                            LedgeraTuiCommandValueReference::RawValue { .. } => {}
                            LedgeraTuiCommandValueReference::ShorthandAsStorageReference(
                                data_moniker,
                            ) => {
                                if k.data_monikers.contains_key(data_moniker) {
                                    let data_digest = k.data_monikers.get(data_moniker).unwrap();
                                    match k.cached_client_knowledge.per_data_value.get(data_digest)
                                    {
                                        None => {
                                            return Err(LedgeraTuiKnowledgeError::NoPromiseOfStorageForGivenData);
                                        }
                                        Some(k_data) => {
                                            if k_data.proofs_of_storage.is_empty() {
                                                return Err(LedgeraTuiKnowledgeError::NoPromiseOfStorageForGivenData);
                                            }
                                        }
                                    }
                                } else {
                                    return Err(LedgeraTuiKnowledgeError::UnknownDataMoniker);
                                }
                            }
                            LedgeraTuiCommandValueReference::ShorthandAsRawValue(data_moniker) => {
                                if k.data_monikers.contains_key(data_moniker) {
                                    let data_digest = k.data_monikers.get(data_moniker).unwrap();
                                    if let Some(data_k) =
                                        k.cached_client_knowledge.per_data_value.get(data_digest)
                                    {
                                        if data_k.data_value.is_none() {
                                            return Err(LedgeraTuiKnowledgeError::ConcreteValueAtGivenDataMonikerUnknown);
                                        }
                                    } else {
                                        return Err(LedgeraTuiKnowledgeError::ConcreteValueAtGivenDataMonikerUnknown);
                                    }
                                } else {
                                    return Err(LedgeraTuiKnowledgeError::UnknownDataMoniker);
                                }
                            }
                        },
                        LedgeraTuiCommandOperationArgument::Predicate(_) => {}
                    }
                }
            }
            LedgeraTuiCommand::PushArg {
                comp_moniker,
                arg_potential_indices,
                data_moniker,
            } => {
                if !k.computations_monikers.contains_key(comp_moniker) {
                    return Err(LedgeraTuiKnowledgeError::UnknownComputationMoniker);
                }
                let comp_id = k.computations_monikers.get(comp_moniker).unwrap();
                if !k
                    .cached_client_knowledge
                    .per_function_instance
                    .contains_key(comp_id)
                {
                    return Err(LedgeraTuiKnowledgeError::MissingComputationInstanceClientKnowledgePleaseRefresh);
                }
                let knowledge_of_comp_instance = k
                    .cached_client_knowledge
                    .per_function_instance
                    .get(comp_id)
                    .unwrap();
                if knowledge_of_comp_instance.spec.is_none() {
                    return Err(LedgeraTuiKnowledgeError::DoesNotHaveSpecOfComputationInstance);
                }
                let comp_spec = &knowledge_of_comp_instance.spec.clone().unwrap();
                if arg_potential_indices
                    .iter()
                    .any(|index| *index >= (comp_spec.arguments.len() as u32))
                {
                    return Err(LedgeraTuiKnowledgeError::IndexOutsideArityOfOperator);
                }
                if let Some(data_digest) = k.data_monikers.get(data_moniker) {
                    if let Some(data_k) = k.cached_client_knowledge.per_data_value.get(data_digest)
                    {
                        if data_k.proofs_of_storage.is_empty() {
                            return Err(
                                LedgeraTuiKnowledgeError::DataMonikerUsedAsArgWithoutProofOfStorage,
                            );
                        }
                    } else {
                        return Err(
                            LedgeraTuiKnowledgeError::DataMonikerUsedAsArgWithoutProofOfStorage,
                        );
                    }
                } else {
                    return Err(LedgeraTuiKnowledgeError::UnknownDataMoniker);
                }
            }
            _ => {}
        }
        Ok(())
    }
}
