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

use std::collections::{BTreeSet, HashMap, HashSet};

use ledgera_pki::manager::SerdeSerializable64BitsSignature;
use ledgera_types::{
    digest::LedgeraDigest, transactions::LedgeraTransaction, votes::vsto::PersistentDataKind,
};

enum ComputationInstanceStateInLog {
    // we have deliveted a Tfun and we need a Tins filling-in all unknown arguments
    DeclaredAndNeedsTins(BTreeSet<u32>),
    // we have deliveted a Tfun and we don't need a Tins
    DeclaredDoesNotNeedTins,
    // we have delivered a Tins with a certain digest and we need a Tout that refers to this digest
    AfterTinsDelivered(LedgeraDigest),
    // the computation is terminated, we do not accept any more transactions referring to that computation instance id
    Terminated,
}

pub struct ValidHistoryOfTransactionsVerifier {
    // so that no two Tarch on the same data hash value are delivered
    // we store the digests of all delivered Tarch
    stored_values: HashSet<(SerdeSerializable64BitsSignature, PersistentDataKind)>,

    // so that the sequence of delivered transactions for a given computation instance id is coherent
    // i.e. in the order i.e.:
    // - either : "Tfun -> Tins(with all unknown from Tfun filled-in) -> Tout(with ref to Tins)"
    // - or     : "Tfun -> Tout(with no ref to Tins if no unknowns in Tfun)"
    computation_instances: HashMap<SerdeSerializable64BitsSignature, ComputationInstanceStateInLog>,
}

pub enum CurrentTransactionValidityGivenHistory {
    // in the case of Tfun, Tins, Tout, we put Some(function_instance_id)
    IsValid(Option<SerdeSerializable64BitsSignature>),
    IsNotValid,
    MayBeAddedInTheFuture(SerdeSerializable64BitsSignature),
}

impl ValidHistoryOfTransactionsVerifier {
    pub fn new() -> Self {
        Self {
            stored_values: HashSet::new(),
            computation_instances: HashMap::new(),
        }
    }

    pub fn try_add_transaction(
        &mut self,
        tx: &LedgeraTransaction,
    ) -> CurrentTransactionValidityGivenHistory {
        match tx {
            LedgeraTransaction::Tsto(anchored_pos) => {
                let tuple = (
                    anchored_pos.v.function_instance_identifier.clone(),
                    anchored_pos.v.data_kind.clone(),
                );
                if self.stored_values.contains(&tuple) {
                    CurrentTransactionValidityGivenHistory::IsNotValid
                } else {
                    self.stored_values.insert(tuple);
                    CurrentTransactionValidityGivenHistory::IsValid(None)
                }
            }
            LedgeraTransaction::Tfun(anchored_pod) => {
                if self
                    .computation_instances
                    .contains_key(&anchored_pod.v.function_instance_identifier)
                {
                    CurrentTransactionValidityGivenHistory::IsNotValid
                } else {
                    if anchored_pod.v.unknown_arguments_indices.is_empty() {
                        self.computation_instances.insert(
                            anchored_pod.v.function_instance_identifier.clone(),
                            ComputationInstanceStateInLog::DeclaredDoesNotNeedTins,
                        );
                    } else {
                        self.computation_instances.insert(
                            anchored_pod.v.function_instance_identifier.clone(),
                            ComputationInstanceStateInLog::DeclaredAndNeedsTins(
                                anchored_pod.v.unknown_arguments_indices.clone(),
                            ),
                        );
                    }
                    CurrentTransactionValidityGivenHistory::IsValid(Some(
                        anchored_pod.v.function_instance_identifier.clone(),
                    ))
                }
            }
            LedgeraTransaction::Tins(anchored_pouav) => {
                if let Some(current_state) = self
                    .computation_instances
                    .get_mut(&anchored_pouav.v.function_instance_identifier)
                {
                    match current_state {
                        ComputationInstanceStateInLog::DeclaredAndNeedsTins(
                            mising_arguments_indices,
                        ) => {
                            let filled_in_args: BTreeSet<u32> = anchored_pouav
                                .v
                                .proposed_unknowns_assignment
                                .keys()
                                .cloned()
                                .collect();
                            if &filled_in_args == mising_arguments_indices {
                                let tins_digest =
                                    LedgeraDigest::from_serializable(&anchored_pouav.v).unwrap();
                                *current_state =
                                    ComputationInstanceStateInLog::AfterTinsDelivered(tins_digest);
                                CurrentTransactionValidityGivenHistory::IsValid(Some(
                                    anchored_pouav.v.function_instance_identifier.clone(),
                                ))
                            } else {
                                CurrentTransactionValidityGivenHistory::IsNotValid
                            }
                        }
                        _ => CurrentTransactionValidityGivenHistory::IsNotValid,
                    }
                } else {
                    CurrentTransactionValidityGivenHistory::MayBeAddedInTheFuture(
                        anchored_pouav.v.function_instance_identifier.clone(),
                    )
                }
            }
            LedgeraTransaction::Tout(anchored_poi) => {
                if let Some(current_state) = self
                    .computation_instances
                    .get_mut(&anchored_poi.v.function_instance_identifier)
                {
                    match current_state {
                        ComputationInstanceStateInLog::DeclaredDoesNotNeedTins => {
                            if anchored_poi.v.unknowns_agreement_ref.is_none() {
                                *current_state = ComputationInstanceStateInLog::Terminated;
                                CurrentTransactionValidityGivenHistory::IsValid(Some(
                                    anchored_poi.v.function_instance_identifier.clone(),
                                ))
                            } else {
                                CurrentTransactionValidityGivenHistory::IsNotValid
                            }
                        }
                        ComputationInstanceStateInLog::DeclaredAndNeedsTins(_) => {
                            if anchored_poi.v.unknowns_agreement_ref.is_none() {
                                CurrentTransactionValidityGivenHistory::IsNotValid
                            } else {
                                CurrentTransactionValidityGivenHistory::MayBeAddedInTheFuture(
                                    anchored_poi.v.function_instance_identifier.clone(),
                                )
                            }
                        }
                        ComputationInstanceStateInLog::AfterTinsDelivered(tins_digest) => {
                            if let Some(anchored_tins_digest) =
                                &anchored_poi.v.unknowns_agreement_ref
                            {
                                if anchored_tins_digest == tins_digest {
                                    *current_state = ComputationInstanceStateInLog::Terminated;
                                    CurrentTransactionValidityGivenHistory::IsValid(Some(
                                        anchored_poi.v.function_instance_identifier.clone(),
                                    ))
                                } else {
                                    CurrentTransactionValidityGivenHistory::IsNotValid
                                }
                            } else {
                                CurrentTransactionValidityGivenHistory::IsNotValid
                            }
                        }
                        ComputationInstanceStateInLog::Terminated => {
                            CurrentTransactionValidityGivenHistory::IsNotValid
                        }
                    }
                } else {
                    CurrentTransactionValidityGivenHistory::MayBeAddedInTheFuture(
                        anchored_poi.v.function_instance_identifier.clone(),
                    )
                }
            }
        }
    }
}
