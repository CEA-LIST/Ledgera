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
use ledgera_pki::{manager::PublicKeyInfrastructure, message::SignatureEntry};
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::proofs::proof_of_storage::ProofOfShipmentToStorage;
use ledgera_types::{
    digest::LedgeraDigest, messages::qval::LedgeraQueryValue, messages::rval::LedgeraResponseValue,
};
use std::sync::Arc;

use crate::topics::LedgeraCoreQueryTopics;

pub async fn retrieve_data_from_storage<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
>(
    service: &Arc<LAT>,
    comm_api_ref: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
    pos: &ProofOfShipmentToStorage,
) -> LAT::Data {
    // The Proof of Shipment to Storage guarantees that at least one honest storer holds
    // this value and is obligated to respond. We therefore retry indefinitely — a failed
    // attempt means the network is temporarily unavailable, not that the data is gone.
    loop {
        let (value_sender, value_receiver) = tokio::sync::oneshot::channel();
        if let Err(e) = query_data_from_storage::<PKI, Sess, LAT>(
            service,
            comm_api_ref.clone(),
            params.clone(),
            pos.v.data_digest.clone(),
            Some(pos.clone()),
            value_sender,
        )
        .await
        {
            log::warn!(
                "failed to issue storage query for digest {} (network error: {:?}), retrying in 1s",
                pos.v.data_digest.to_hexadecimal_string(),
                e
            );
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            continue;
        }
        match value_receiver.await {
            Ok(Some(v)) => return v,
            Ok(None) => {
                log::warn!(
                    "no storer returned a value matching digest {} within the query window, retrying in 1s",
                    pos.v.data_digest.to_hexadecimal_string()
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            Err(_) => {
                log::warn!(
                    "storage query response channel closed before receiving a value for digest {} \
                     (query task may have been cancelled), retrying in 1s",
                    pos.v.data_digest.to_hexadecimal_string()
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
}

/**
 A function that might be used by both "computer nodes" and "client nodes" to retrieve
data from the storage.
 **/
pub async fn query_data_from_storage<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
>(
    service: &Arc<LAT>,
    comm_api_ref: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
    data_digest: LedgeraDigest,
    pos_opt: Option<ProofOfShipmentToStorage>,
    matching_data_sender: tokio::sync::oneshot::Sender<Option<LAT::Data>>,
) -> Result<(), LedgeraCommunicationError<Sess::CommRuntimeError>> {
    let (sender, receiver) = tokio::sync::mpsc::channel(128);
    {
        let mut comm_api = comm_api_ref.lock().await;
        comm_api
            .query_network::<LedgeraQueryValue, LedgeraResponseValue<LAT::Data>>(
                &params,
                &LedgeraCoreQueryTopics::Value.get_query_topic_str(service.as_ref()),
                &LedgeraQueryValue::new(data_digest.clone(), pos_opt),
                sender,
            )
            .await?
    }
    tool_function_handle_responses_to_data_query::<LAT::Data>(
        receiver,
        data_digest,
        matching_data_sender,
    );
    Ok(())
}

fn tool_function_handle_responses_to_data_query<
    DataValue: serde::Serialize + for<'a> serde::Deserialize<'a> + Send + 'static,
>(
    mut receiver: tokio::sync::mpsc::Receiver<(LedgeraResponseValue<DataValue>, SignatureEntry)>,
    data_digest: LedgeraDigest,
    matching_data_sender: tokio::sync::oneshot::Sender<Option<DataValue>>,
) {
    tokio::spawn(async move {
        let mut found_correct_response = None;
        'wait_for_next_response: loop {
            match tokio::time::timeout(tokio::time::Duration::from_secs(5), receiver.recv()).await {
                Err(e) => {
                    log::warn!(
                        "timeout '{:?}' exceeded when waiting for a response to a data query for digest {:?}",
                        e,
                        data_digest
                    );
                    break 'wait_for_next_response;
                }
                Ok(None) => {
                    break 'wait_for_next_response;
                }
                Ok(Some((payload, signature_entry))) => match payload {
                    LedgeraResponseValue::Value(v) => match LedgeraDigest::from_serializable(&v) {
                        Ok(d) => {
                            if d == data_digest {
                                found_correct_response = Some(v);
                                break 'wait_for_next_response;
                            } else {
                                log::warn!(
                                            "data value received from public key {:?} as a response to data query on digest {:?} does not have the same digest : {:?}",
                                            signature_entry.serialized_signing_public_key,
                                            data_digest,
                                            d
                                        );
                            }
                        }
                        Err(e) => {
                            log::warn!(
                                        "could not make a digest out of data value received from public key {:?} as a response to data query on digest {:?} with error {:?}",
                                        signature_entry.serialized_signing_public_key,
                                        data_digest,
                                        e
                                    );
                        }
                    },
                    LedgeraResponseValue::NoValue => {
                        log::warn!(
                            "received empty data response from public key {:?} for digest {:?}",
                            signature_entry.serialized_signing_public_key,
                            data_digest
                        );
                    }
                },
            }
        }
        let _ = matching_data_sender.send(found_correct_response);
    });
}
