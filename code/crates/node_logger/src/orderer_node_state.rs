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

use ledgera_comms::comm_api::{
    LedgeraInternalCommunicationInterface, LedgeraInternalCommunicationParameters,
};
use ledgera_comms::comm_session::PubSubNetwork;
use ledgera_comms::error::LedgeraCommunicationError;
use ledgera_core_logic::roles::LedgeraCoreRoles;
use ledgera_core_logic::topics::{LedgeraCorePublicationTopics, LedgeraCoreQueryTopics};
use ledgera_pki::manager::PublicKeyInfrastructure;
use ledgera_pki::message::AuthenticatableMessage;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::messages::deliver::LedgeraTransactionDeliveryNotification;
use ledgera_types::messages::qaud::LedgeraQueryAudit;
use ledgera_types::messages::raud::LedgeraResponseAudit;
use ledgera_types::traits::LedgeraQuorumContainingMessage;
use ledgera_types::transactions::LedgeraTransaction;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::centralized_log::LedgeraProvisionalCentralizedLog;

pub struct LedgeraOrdererNodeState<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
> {
    comm_session: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    comm_params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
    service: Arc<LAT>,
    // for now the log is a simple list
    local_log: Arc<Mutex<LedgeraProvisionalCentralizedLog>>,
    //
    handle_submitted_transactions_task: Option<tokio::task::JoinHandle<()>>,
    handle_log_audit_queries_task: Option<tokio::task::JoinHandle<()>>,
}

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork, LAT: LedgeraApplicationTemplate>
    LedgeraOrdererNodeState<PKI, Sess, LAT>
{
    pub fn new(
        comm_session: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
        comm_params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
        service: Arc<LAT>,
    ) -> Self {
        Self {
            comm_session,
            comm_params,
            service,
            local_log: Arc::new(Mutex::new(LedgeraProvisionalCentralizedLog::new())),
            handle_submitted_transactions_task: None,
            handle_log_audit_queries_task: None,
        }
    }

    pub async fn abort(&mut self) {
        if let Some(x) = &self.handle_submitted_transactions_task {
            log::warn!(
                "As {:?} : aborting handle_submitted_transactions_task",
                LedgeraCoreRoles::SecureLogger
            );
            x.abort();
        }
        if let Some(x) = &self.handle_log_audit_queries_task {
            log::warn!(
                "As {:?} : aborting handle_log_audit_queries_task",
                LedgeraCoreRoles::SecureLogger
            );
            x.abort();
        }
    }

    pub async fn run(&mut self) -> Result<(), LedgeraCommunicationError<Sess::CommRuntimeError>> {
        // channels to receive transactions submitted by voters
        let (tx_sender, mut tx_receiver) = tokio::sync::mpsc::channel(128);

        // channels to receive audit queries submitted by clients
        let (audit_queries_sender, mut audit_queries_receiver) = tokio::sync::mpsc::channel(128);

        {
            let mut comm_sess = self.comm_session.lock().await;
            // subsribe to pertinent topic to receive transactions from voters
            match comm_sess
                .subscribe_to_topic_and_deserialize_as::<LedgeraTransaction>(
                    &self.comm_params,
                    &LedgeraCorePublicationTopics::TransactionSubmission
                        .get_publication_topic_str(self.service.as_ref()),
                    tx_sender,
                )
                .await
            {
                Ok(_) => {
                    log::info!(
                        "As {:?} : subscribed to submitted transactions",
                        LedgeraCoreRoles::SecureLogger
                    );
                }
                Err(e) => {
                    return Err(e);
                }
            }
            //  declare queryable on stored data to receive data queries from voters/clients
            match comm_sess
                .declare_queryable::<LedgeraQueryAudit, LedgeraResponseAudit>(
                    &self.comm_params.clone(),
                    &LedgeraCoreQueryTopics::Audit.get_query_topic_str(self.service.as_ref()),
                    audit_queries_sender,
                )
                .await
            {
                Ok(_) => {
                    log::info!(
                        "As {:?} : declared queryable to answer log audit queries",
                        LedgeraCoreRoles::SecureLogger
                    );
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        {
            let comm_params_ref = self.comm_params.clone();
            let comm_session_ref = self.comm_session.clone();
            let service_ref = self.service.clone();
            let local_log_ref = self.local_log.clone();
            self.handle_submitted_transactions_task = Some(tokio::spawn(async move {
                while let Some((submitted_transaction, _)) = tx_receiver.recv().await {
                    match submitted_transaction.verify_vote_quorums::<PKI>(
                        &comm_params_ref.known_participants,
                        comm_params_ref.byzantine_threshold,
                    ) {
                        Ok(_) => {
                            let delivered_txs_notifications = {
                                let mut local_log = local_log_ref.lock().await;
                                local_log.process_submitted_transaction(submitted_transaction)
                            };
                            {
                                let mut comm_sess = comm_session_ref.lock().await;
                                for dlvrd_tx in delivered_txs_notifications {
                                    if let Err(e) = comm_sess
                                        .serialize_and_publish_on_topic::<LedgeraTransactionDeliveryNotification>(
                                            &comm_params_ref,
                                            &LedgeraCorePublicationTopics::TransactionDelivery.get_publication_topic_str(service_ref.as_ref()),
                                            &dlvrd_tx
                                        ).await
                                    {
                                        log::warn!(
                                            "As {:?} : could not emit transaction delivery notification with error : {:?}",
                                            LedgeraCoreRoles::SecureLogger,
                                            e
                                        )
                                    } else {
                                        log::info!(
                                            "As {:?} : emitted transaction delivery notification",
                                            LedgeraCoreRoles::SecureLogger
                                        )
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!(
                                "As {:?} : received a transaction with incorrect quorum(s) inside : {:?}",
                                LedgeraCoreRoles::SecureLogger,
                                e
                            )
                        }
                    }
                }
                log::warn!(
                    "As {:?} : handle_submitted_transactions_task terminated",
                    LedgeraCoreRoles::SecureLogger
                );
            }));
        }

        {
            let local_log_lock_ref = self.local_log.clone();
            let comm_params_ref = self.comm_params.clone();
            self.handle_log_audit_queries_task = Some(tokio::spawn(async move {
                while let Some((backend_query, ledgera_query_content)) =
                    audit_queries_receiver.recv().await
                {
                    let response = {
                        let local_log = local_log_lock_ref.lock().await;
                        local_log.respond_to_audit_query(&ledgera_query_content)
                    };
                    if let Err(e) = Sess::reply_to_incoming_query(
                        &backend_query,
                        AuthenticatableMessage::create::<LedgeraResponseAudit, PKI>(
                            &response,
                            &comm_params_ref.signing_key,
                        )
                        .unwrap(),
                    )
                    .await
                    {
                        log::warn!(
                            "As {:?} : could not respond to log audit query with error {:?}",
                            LedgeraCoreRoles::SecureLogger,
                            e
                        );
                    }
                }
                log::warn!(
                    "As {:?} : handle_log_audit_queries_task terminating",
                    LedgeraCoreRoles::SecureLogger
                );
            }));
        }

        /*if let Some(handle_submitted_transactions_task) = self.handle_submitted_transactions_task {
            handle_submitted_transactions_task.await;
        }*/
        Ok(())
    }
}
