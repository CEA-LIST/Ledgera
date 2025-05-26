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
use ledgera_types::digest::LedgeraDigest;
use ledgera_types::messages::qval::LedgeraQueryValue;
use ledgera_types::messages::rval::LedgeraResponseValue;
use ledgera_types::proofs::proof_of_storage::ProofOfShipmentToStorage;
use ledgera_types::requests::rsto::LedgeraServerSideStorageRequest;
use ledgera_types::traits::LedgeraQuorumContainingMessage;
use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::watch;
use tokio::sync::Mutex;

pub struct LedgeraStorageNodeState<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
> {
    comm_session: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    comm_params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
    service: Arc<LAT>,
    // for now the storage is a simple local dictionnary
    local_storage: Arc<Mutex<HashMap<LedgeraDigest, LAT::Data>>>,
    // notifies waiting query tasks whenever a new value is inserted into local_storage
    storage_inserted_tx: Arc<watch::Sender<()>>,
    //
    handle_storage_requests_task: Option<tokio::task::JoinHandle<()>>,
    handle_data_queries_task: Option<tokio::task::JoinHandle<()>>,
}

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork, LAT: LedgeraApplicationTemplate>
    LedgeraStorageNodeState<PKI, Sess, LAT>
{
    pub fn new(
        comm_session: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
        comm_params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
        service: Arc<LAT>,
    ) -> Self {
        let (storage_inserted_tx, _) = watch::channel(());
        Self {
            comm_session,
            comm_params,
            service,
            local_storage: Arc::new(Mutex::new(HashMap::new())),
            storage_inserted_tx: Arc::new(storage_inserted_tx),
            handle_storage_requests_task: None,
            handle_data_queries_task: None,
        }
    }

    pub async fn abort(&mut self) {
        if let Some(x) = &self.handle_storage_requests_task {
            log::warn!(
                "As {:?} : aborting handle_storage_requests_task",
                LedgeraCoreRoles::PersistentStorage
            );
            x.abort();
        }
        if let Some(x) = &self.handle_data_queries_task {
            log::warn!(
                "As {:?} : aborting handle_data_queries_task",
                LedgeraCoreRoles::PersistentStorage
            );
            x.abort();
        }
    }

    pub async fn run(&mut self) -> Result<(), LedgeraCommunicationError<Sess::CommRuntimeError>> {
        // channel to receive server-side storage requests from voters
        let (storage_requests_sender, mut storage_requests_receiver) =
            tokio::sync::mpsc::channel(128);

        // channel to receive data queries submitted by voters or clients
        let (data_queries_sender, mut data_queries_receiver) = tokio::sync::mpsc::channel(128);

        {
            let mut comm_sess = self.comm_session.lock().await;
            // subscribe to the pertinent topic to receive server-side storage requests from voters
            match comm_sess.subscribe_to_topic_and_deserialize_as::<
                LedgeraServerSideStorageRequest<LAT::Data>
            >(
                &self.comm_params,
                &LedgeraCorePublicationTopics::Rsto.get_publication_topic_str(self.service.as_ref()),
                storage_requests_sender
            ).await {
                Ok(_) => {
                    log::info!(
                        "As {:?} : subscribed to server-side storage requests",
                        LedgeraCoreRoles::PersistentStorage
                    );
                }
                Err(e) => {
                    return Err(e);
                }
            }
            //  declare queryable on stored data to receive data queries from voters/clients
            match comm_sess
                .declare_queryable::<LedgeraQueryValue, LedgeraResponseValue<LAT::Data>>(
                    &self.comm_params.clone(),
                    &LedgeraCoreQueryTopics::Value.get_query_topic_str(self.service.as_ref()),
                    data_queries_sender,
                )
                .await
            {
                Ok(_) => {
                    log::info!(
                        "As {:?} : declared queryable to answer data queries",
                        LedgeraCoreRoles::PersistentStorage
                    );
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        {
            let comm_params_ref = self.comm_params.clone();
            let storage_lock_ref = self.local_storage.clone();
            let storage_notifier = self.storage_inserted_tx.clone();
            self.handle_storage_requests_task = Some(tokio::spawn(async move {
                while let Some((storage_request, _)) = storage_requests_receiver.recv().await {
                    match storage_request.verify_vote_quorums::<PKI>(
                        &comm_params_ref.known_participants,
                        comm_params_ref.byzantine_threshold,
                    ) {
                        Ok(_) => {
                            match LedgeraDigest::from_serializable(&storage_request.data_value) {
                                Ok(value_digest) => {
                                    let log_hex_str = value_digest.to_hexadecimal_string();
                                    let mut storage = storage_lock_ref.lock().await;
                                    if let Vacant(v) = storage.entry(value_digest) {
                                        log::info!(
                                            "As {:?} : inserting new value in local copy of the storage at digest {:}",
                                            LedgeraCoreRoles::PersistentStorage,
                                            log_hex_str
                                        );
                                        v.insert(storage_request.data_value);
                                        // wake any tasks waiting for a newly stored value
                                        storage_notifier.send(()).ok();
                                    } else {
                                        log::info!(
                                            "As {:?} : value to insert already in local copy of the storage at digest {:}",
                                            LedgeraCoreRoles::PersistentStorage,
                                            log_hex_str
                                        );
                                    }
                                }
                                Err(e) => {
                                    log::warn!(
                                        "As {:?} : could not compute digest of value received in storage request : {:?}",
                                        LedgeraCoreRoles::PersistentStorage,
                                        e
                                    )
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!(
                                "As {:?} : received a storage request with an incorrect quorum : {:?}",
                                LedgeraCoreRoles::PersistentStorage,
                                e
                            )
                        }
                    }
                }
                log::warn!(
                    "As {:?} : handle_storage_requests_task terminated",
                    LedgeraCoreRoles::PersistentStorage
                );
            }));
        }

        {
            let comm_params_ref = self.comm_params.clone();
            let storage_lock_ref = self.local_storage.clone();
            let storage_inserted_rx = self.storage_inserted_tx.subscribe();
            self.handle_data_queries_task = Some(tokio::spawn(async move {
                let mut inner_handles = vec![];
                while let Some((backend_query, ledgera_query_content)) =
                    data_queries_receiver.recv().await
                {
                    inner_handles.retain(|h: &tokio::task::JoinHandle<()>| !h.is_finished());
                    let storage_inserted_rx_clone = storage_inserted_rx.clone();
                    let query_answered = tool_function_answer_to_data_query::<PKI, Sess, LAT>(
                        &comm_params_ref,
                        storage_lock_ref.clone(),
                        storage_inserted_rx_clone,
                        backend_query,
                        ledgera_query_content.digest_of_value,
                        ledgera_query_content.pos_opt,
                    )
                    .await;
                    if let Some(inner_task) = query_answered {
                        inner_handles.push(inner_task);
                    }
                }
                log::warn!(
                    "As {:?} : handle_data_queries_task terminating",
                    LedgeraCoreRoles::PersistentStorage
                );
                for x in inner_handles {
                    let _ = x.await;
                }
                log::warn!(
                    "As {:?} : handle_data_queries_task terminated",
                    LedgeraCoreRoles::PersistentStorage
                );
            }));
        }

        Ok(())
    }
}

async fn tool_function_answer_to_data_query<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
>(
    comm_params: &Arc<LedgeraInternalCommunicationParameters<PKI>>,
    local_storage: Arc<Mutex<HashMap<LedgeraDigest, LAT::Data>>>,
    storage_inserted_rx: watch::Receiver<()>,
    query: Sess::IncomingQuery,
    data_digest: LedgeraDigest,
    opt_promise_of_storage: Option<ProofOfShipmentToStorage>,
) -> Option<tokio::task::JoinHandle<()>> {
    let local_storage_clone = local_storage.clone();
    let found_in_storage = {
        let storage = local_storage.lock().await;
        storage.get(&data_digest).cloned()
    };
    match found_in_storage {
        Some(got_data) => {
            log::info!(
                "As {:?} : queried value is already in local copy of the storage : answering now",
                LedgeraCoreRoles::PersistentStorage
            );
            let response: LedgeraResponseValue<LAT::Data> = LedgeraResponseValue::Value(got_data);
            if let Err(e) = Sess::reply_to_incoming_query(
                &query,
                AuthenticatableMessage::create::<LedgeraResponseValue<LAT::Data>, PKI>(
                    &response,
                    &comm_params.signing_key,
                )
                .unwrap(),
            )
            .await
            {
                log::warn!(
                    "As {:?} : could not respond to data query with error {:?}",
                    LedgeraCoreRoles::PersistentStorage,
                    e
                );
            }
            None
        }
        None => {
            let answer_now: Option<&'static str> = match opt_promise_of_storage {
                None => Some("no promise of storage was provided"),
                Some(pos) => {
                    match pos.verify_proof_of_shipment_to_storage::<PKI>(
                        &comm_params.known_participants,
                        comm_params.byzantine_threshold,
                    ) {
                        Ok(_) => {
                            // We must have "lps.v_sto.h = h" i.e. the LP_S must certify the
                            // same digest as the one being queried, not just be internally valid.
                            if pos.v.data_digest != data_digest {
                                log::warn!(
                                    "As {:?} : promise of storage certifies digest {:?} but query is for digest {:?}; treating as invalid",
                                    LedgeraCoreRoles::PersistentStorage,
                                    pos.v.data_digest,
                                    data_digest
                                );
                                Some("promise of storage is for a different digest than the one queried")
                            } else {
                                None
                            }
                        }
                        Err(e) => {
                            log::warn!(
                                "As {:?} : received an incorrect promise of storage for query on data digest {:?} with error {:?}",
                                LedgeraCoreRoles::PersistentStorage,
                                data_digest,
                                e
                            );
                            Some("incorrect promise of storage was provided")
                        }
                    }
                }
            };
            match answer_now {
                Some(reason) => {
                    log::info!(
                        "As {:?} : queried value is not in the local copy of the storage : answering now negatively because {:}",
                        LedgeraCoreRoles::PersistentStorage,
                        reason
                    );
                    let response: LedgeraResponseValue<LAT::Data> = LedgeraResponseValue::NoValue;
                    if let Err(e) = Sess::reply_to_incoming_query(
                        &query,
                        AuthenticatableMessage::create::<LedgeraResponseValue<LAT::Data>, PKI>(
                            &response,
                            &comm_params.signing_key,
                        )
                        .unwrap(),
                    )
                    .await
                    {
                        log::warn!(
                            "As {:?} : could not respond to data query with error {:?}",
                            LedgeraCoreRoles::PersistentStorage,
                            e
                        );
                    }
                    None
                }
                None => {
                    // spawn a new task in which one waits for the value to be
                    // inserted in the local copy of the storage before answering
                    Some(tool_function_respond_with_data_once_in_store::<
                        PKI,
                        Sess,
                        LAT,
                    >(
                        &comm_params.signing_key,
                        local_storage_clone,
                        storage_inserted_rx,
                        query,
                        data_digest,
                    ))
                }
            }
        }
    }
}

fn tool_function_respond_with_data_once_in_store<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
>(
    private_key: &Arc<PKI::SigningKey>,
    local_storage: Arc<Mutex<HashMap<LedgeraDigest, LAT::Data>>>,
    mut storage_inserted_rx: watch::Receiver<()>,
    query: Sess::IncomingQuery,
    data_digest: LedgeraDigest,
) -> tokio::task::JoinHandle<()> {
    let private_key_ref = private_key.clone();
    tokio::spawn(async move {
        loop {
            {
                let storage = local_storage.lock().await;
                if let Some(got_data) = storage.get(&data_digest).cloned() {
                    log::info!(
                        "As {:?} : responding to queried value with delay now that the promised value is in local copy of the storage at digest {:}",
                        LedgeraCoreRoles::PersistentStorage,
                        data_digest.to_hexadecimal_string()
                    );
                    let response: LedgeraResponseValue<LAT::Data> =
                        LedgeraResponseValue::Value(got_data);
                    if let Err(e) = Sess::reply_to_incoming_query(
                        &query,
                        AuthenticatableMessage::create::<LedgeraResponseValue<LAT::Data>, PKI>(
                            &response,
                            &private_key_ref,
                        )
                        .unwrap(),
                    )
                    .await
                    {
                        log::warn!(
                            "As {:?} : could not respond to data query with error {:?}",
                            LedgeraCoreRoles::PersistentStorage,
                            e
                        );
                    }
                    return;
                }
            }
            // wait for any new insertion into local_storage before re-checking
            if storage_inserted_rx.changed().await.is_err() {
                log::warn!(
                    "As {:?} : storage task terminated before promised value arrived at digest {:}; dropping query",
                    LedgeraCoreRoles::PersistentStorage,
                    data_digest.to_hexadecimal_string()
                );
                return;
            }
        }
    })
}
