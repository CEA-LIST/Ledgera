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

use ledgera_core_logic::roles::LedgeraCoreRoles;
use ledgera_pki::manager::SerdeSerializable64BitsSignature;
use ledgera_types::messages::{
    deliver::LedgeraTransactionDeliveryNotification, qaud::LedgeraQueryAudit,
    raud::LedgeraResponseAudit,
};
use ledgera_types::transactions::LedgeraTransaction;
use std::collections::{HashMap, VecDeque};

use crate::valid_history::{
    CurrentTransactionValidityGivenHistory, ValidHistoryOfTransactionsVerifier,
};

pub struct LedgeraProvisionalCentralizedLog {
    log: Vec<LedgeraTransaction>,

    buffered_transactions: HashMap<SerdeSerializable64BitsSignature, VecDeque<LedgeraTransaction>>,

    valid_history_verifier: ValidHistoryOfTransactionsVerifier,
}

impl LedgeraProvisionalCentralizedLog {
    pub fn new() -> Self {
        Self {
            log: vec![],
            buffered_transactions: HashMap::new(),
            valid_history_verifier: ValidHistoryOfTransactionsVerifier::new(),
        }
    }

    pub fn respond_to_audit_query(&self, query: &LedgeraQueryAudit) -> LedgeraResponseAudit {
        match query {
            LedgeraQueryAudit::StoredValue(value_digest) => {
                for tx in &self.log {
                    match tx {
                        LedgeraTransaction::Tsto(anchored_pos)
                            if anchored_pos.v.data_digest == *value_digest =>
                        {
                            return LedgeraResponseAudit::new(vec![tx.clone()]);
                        }
                        _ => {}
                    }
                }
                LedgeraResponseAudit::new(vec![])
            }
            LedgeraQueryAudit::Computation(computation_id) => {
                let mut txs = vec![];
                for tx in &self.log {
                    match tx {
                        LedgeraTransaction::Tfun(anchored_pood)
                            if anchored_pood.v.function_instance_identifier == *computation_id =>
                        {
                            txs.push(tx.clone());
                        }
                        LedgeraTransaction::Tins(anchored_pouav)
                            if anchored_pouav.v.function_instance_identifier == *computation_id =>
                        {
                            txs.push(tx.clone());
                        }
                        LedgeraTransaction::Tout(anchored_poi)
                            if anchored_poi.v.function_instance_identifier == *computation_id =>
                        {
                            txs.push(tx.clone());
                        }
                        _ => {}
                    }
                }
                LedgeraResponseAudit::new(txs)
            }
        }
    }

    pub fn process_submitted_transaction(
        &mut self,
        tx: LedgeraTransaction,
    ) -> Vec<LedgeraTransactionDeliveryNotification> {
        let mut notifications = Vec::new();
        match self.valid_history_verifier.try_add_transaction(&tx) {
            CurrentTransactionValidityGivenHistory::IsValid(opt_comp_instance_id) => {
                {
                    let delivered_at_index = self.log.len();
                    log::info!(
                        "As {:?} : delivering newly received '{:}' transaction at index {:}",
                        LedgeraCoreRoles::SecureLogger,
                        tx.get_transaction_kind(),
                        delivered_at_index
                    );
                    self.log.push(tx.clone());
                    notifications.push(LedgeraTransactionDeliveryNotification::new(
                        delivered_at_index as u32,
                        tx,
                    ));
                }
                if let Some(function_instance_id) = opt_comp_instance_id {
                    // the fact that we added a transaction to the history might make so that other buffered transactions
                    // might also be added now
                    // e.g., if a "Tins" transaction has been added and there was a buffered "Tout"
                    'try_add_another: loop {
                        if let Some(buffered_transactions) =
                            self.buffered_transactions.remove(&function_instance_id)
                        {
                            log::info!(
                                "As {:?} : might trigger delivery of other transactions for computation instance {:}",
                                LedgeraCoreRoles::SecureLogger,
                                function_instance_id.to_hexadecimal_string()
                            );
                            let (opt_other_delivered_tx_notif, opt_new_buffer) = self
                                .filter_out_buffered_computation_instance_transactions(
                                    &function_instance_id,
                                    buffered_transactions,
                                );
                            if let Some(remaining_buffered_transactions) = opt_new_buffer {
                                self.buffered_transactions.insert(
                                    function_instance_id.clone(),
                                    remaining_buffered_transactions,
                                );
                            }
                            if let Some(other_delivered_tx_notif) = opt_other_delivered_tx_notif {
                                notifications.push(other_delivered_tx_notif);
                            } else {
                                break 'try_add_another;
                            }
                        } else {
                            // there are no buffered transactions
                            log::info!(
                                "As {:?} : no other deliverable buffered transactions for computation instance {:}",
                                LedgeraCoreRoles::SecureLogger,
                                function_instance_id.to_hexadecimal_string()
                            );
                            break 'try_add_another;
                        }
                    }
                }
            }
            CurrentTransactionValidityGivenHistory::IsNotValid => {
                // do nothing
            }
            CurrentTransactionValidityGivenHistory::MayBeAddedInTheFuture(comp_instance_id) => {
                let buffered = self
                    .buffered_transactions
                    .entry(comp_instance_id)
                    .or_default();
                buffered.push_back(tx);
            }
        }

        notifications
    }

    fn filter_out_buffered_computation_instance_transactions(
        &mut self,
        function_instance_id: &SerdeSerializable64BitsSignature,
        mut buffered_transactions: VecDeque<LedgeraTransaction>,
    ) -> (
        Option<LedgeraTransactionDeliveryNotification>,
        Option<VecDeque<LedgeraTransaction>>,
    ) {
        let mut new_buffer = VecDeque::new();
        let mut delivered_tx = None;
        while let Some(next_tx) = buffered_transactions.pop_front() {
            match self.valid_history_verifier.try_add_transaction(&next_tx) {
                CurrentTransactionValidityGivenHistory::IsValid(cmp_id) => {
                    if cmp_id != Some(function_instance_id.clone()) {
                        log::warn!(
                            "As {:?} : state-machine invariant violation: buffered transaction became valid with instance id {:?} but was buffered under {:?}; delivering anyway",
                            LedgeraCoreRoles::SecureLogger,
                            cmp_id,
                            function_instance_id
                        );
                    }
                    let delivered_at_index = self.log.len();
                    log::info!(
                        "As {:?} : delivering buffered '{:}' transaction at index {:}",
                        LedgeraCoreRoles::SecureLogger,
                        next_tx.get_transaction_kind(),
                        delivered_at_index
                    );
                    self.log.push(next_tx.clone());
                    delivered_tx = Some(LedgeraTransactionDeliveryNotification::new(
                        delivered_at_index as u32,
                        next_tx,
                    ));
                    new_buffer.append(&mut buffered_transactions);
                    break;
                }
                CurrentTransactionValidityGivenHistory::IsNotValid => {
                    // nothing, we can drop the transaction
                }
                CurrentTransactionValidityGivenHistory::MayBeAddedInTheFuture(cmp_id) => {
                    if &cmp_id != function_instance_id {
                        log::warn!(
                            "As {:?} : state-machine invariant violation: buffered transaction still pending with instance id {:?} but was buffered under {:?}; dropping transaction",
                            LedgeraCoreRoles::SecureLogger,
                            cmp_id,
                            function_instance_id
                        );
                    } else {
                        // we add it to the back of the new buffer for the next iteration
                        new_buffer.push_back(next_tx);
                    }
                }
            }
        }
        if new_buffer.is_empty() {
            (delivered_tx, None)
        } else {
            (delivered_tx, Some(new_buffer))
        }
    }
}
