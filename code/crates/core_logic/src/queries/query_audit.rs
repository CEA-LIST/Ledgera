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
use ledgera_types::messages::qaud::LedgeraQueryAudit;
use ledgera_types::messages::raud::LedgeraResponseAudit;
use ledgera_types::transactions::LedgeraTransaction;
use std::sync::Arc;

use crate::topics::LedgeraCoreQueryTopics;

/**
 A function that might be used by "client nodes" to retrieve
audit info from the log.
 **/
pub async fn query_audit_from_log<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
>(
    service: &Arc<LAT>,
    comm_api_ref: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
    audit_query: LedgeraQueryAudit,
    audit_response_sender: tokio::sync::oneshot::Sender<Vec<LedgeraTransaction>>,
) -> Result<(), LedgeraCommunicationError<Sess::CommRuntimeError>> {
    let (sender, receiver) = tokio::sync::mpsc::channel(128);
    {
        let mut comm_api = comm_api_ref.lock().await;
        comm_api
            .query_network::<LedgeraQueryAudit, LedgeraResponseAudit>(
                &params,
                &LedgeraCoreQueryTopics::Audit.get_query_topic_str(service.as_ref()),
                &audit_query,
                sender,
            )
            .await?
    }
    tool_function_handle_responses_to_audit_query(receiver, audit_response_sender);
    Ok(())
}

fn tool_function_handle_responses_to_audit_query(
    mut receiver: tokio::sync::mpsc::Receiver<(LedgeraResponseAudit, SignatureEntry)>,
    audit_response_sender: tokio::sync::oneshot::Sender<Vec<LedgeraTransaction>>,
) {
    tokio::spawn(async move {
        let mut gathered_txs = Vec::new();
        // NOTE: In the current research prototype the secure log is a single centralised node,
        // so accepting one response is acceptable for now.
        // In a fully Byzantine-fault-tolerant deployment this function must be extended to:
        //   (1) collect at least f+1 matching responses before trusting any result, so that
        //       at least one honest logger is guaranteed to have contributed;
        //   (2) verify that each returned transaction is correctly anchored in the log and
        //       is indeed a valid answer to the issued query (digest match, signature checks).
        // Both points are deferred until the distributed secure-log layer is implemented.
        match tokio::time::timeout(tokio::time::Duration::from_secs(5), receiver.recv()).await {
            Err(e) => {
                log::warn!(
                    "timeout '{:?}' exceeded when waiting for a response to a audit query",
                    e
                );
            }
            Ok(None) => {}
            Ok(Some((response, _))) => {
                for tx in response.transactions {
                    // NOTE: transaction validity and query relevance are not checked here —
                    // see the comment above for the full list of deferred verifications.
                    gathered_txs.push(tx);
                }
            }
        }
        let _ = audit_response_sender.send(gathered_txs);
    });
}
