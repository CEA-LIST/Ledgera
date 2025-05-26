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

use std::{collections::HashMap, sync::Arc};

use ledgera_comms::{
    comm_api::{LedgeraInternalCommunicationInterface, LedgeraInternalCommunicationParameters},
    comm_session::PubSubNetwork,
};
use ledgera_core_logic::{
    quorum::collect_quorum, roles::LedgeraCoreRoles, topics::LedgeraCorePublicationTopics,
};
use ledgera_pki::{manager::PublicKeyInfrastructure, message::SignatureEntry};
use ledgera_types::{
    app_template::template::LedgeraApplicationTemplate,
    proofs::proof_of_storage::ProofOfShipmentToStorage,
    requests::rsto::LedgeraServerSideStorageRequest, transactions::LedgeraTransaction,
    votes::vsto::LedgeraVoteStored,
};
use tokio::task::JoinSet;

use crate::management::error::VoterComputationBehaviorError;

/// Emits one (Rsto, Vsto) pair per request, then waits for f+1 Vstored quorums
/// and submits the resulting Tsto transactions.
pub async fn emit_and_collect_storage_quorums<PKI, Sess, LAT>(
    comm_api: &Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    comm_params: &Arc<LedgeraInternalCommunicationParameters<PKI>>,
    service: &Arc<LAT>,
    rsto_requests: Vec<(
        LedgeraServerSideStorageRequest<LAT::Data>,
        LedgeraVoteStored,
    )>,
    vstored_receiver: &mut tokio::sync::mpsc::Receiver<(SignatureEntry, LedgeraVoteStored)>,
) -> Result<(), VoterComputationBehaviorError<Sess, LAT>>
where
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
{
    if rsto_requests.is_empty() {
        return Ok(());
    }

    let mut collect_quorums_join_set: JoinSet<(LedgeraVoteStored, Option<_>)> = JoinSet::new();
    let mut vstored_votes_sigs_senders: HashMap<
        LedgeraVoteStored,
        tokio::sync::mpsc::Sender<SignatureEntry>,
    > = HashMap::new();

    for (rsto_request, vsto_vote) in rsto_requests {
        {
            let mut comm_sess = comm_api.lock().await;
            if let Err(e) = comm_sess
                .serialize_and_publish_on_topic::<LedgeraServerSideStorageRequest<LAT::Data>>(
                    comm_params,
                    &LedgeraCorePublicationTopics::Rsto.get_publication_topic_str(service.as_ref()),
                    &rsto_request,
                )
                .await
            {
                log::warn!(
                    "As {:?} : could not emit server-side storage request with error : {:?}",
                    LedgeraCoreRoles::VoterComputer,
                    e
                )
            } else {
                log::info!(
                    "As {:?} : emitted server-side storage request to insert data at digest {}",
                    LedgeraCoreRoles::VoterComputer,
                    vsto_vote.data_digest.to_hexadecimal_string()
                )
            }

            if let Err(e) = comm_sess
                .serialize_and_publish_on_topic::<LedgeraVoteStored>(
                    comm_params,
                    &LedgeraCorePublicationTopics::Vsto.get_publication_topic_str(service.as_ref()),
                    &vsto_vote,
                )
                .await
            {
                log::warn!(
                    "As {:?} : could not emit vote notifying emission of server-side storage request with error : {:?}",
                    LedgeraCoreRoles::VoterComputer,
                    e
                )
            } else {
                log::info!(
                    "As {:?} : emitted vote notification confirming having send a server-side storage request to insert data at digest {:}",
                    LedgeraCoreRoles::VoterComputer,
                    vsto_vote.data_digest.to_hexadecimal_string()
                )
            }
        }

        let (vstored_sig_sender, vstored_sig_receiver) = tokio::sync::mpsc::channel(128);
        {
            let byzantine_threshold = comm_params.byzantine_threshold as usize;
            let vstored_cloned = vsto_vote.clone();
            collect_quorums_join_set.spawn(async move {
                let quorum = collect_quorum::<PKI>(
                    bincode::serialize(&vstored_cloned).unwrap(),
                    vstored_sig_receiver,
                    byzantine_threshold,
                )
                .await;
                (vstored_cloned, quorum)
            });
        }
        vstored_votes_sigs_senders.insert(vsto_vote, vstored_sig_sender);
    }

    'wait_until_all_quorums: loop {
        tokio::select! {
            Some((sig, vstored)) = vstored_receiver.recv() => {
                if let Some(sender) = vstored_votes_sigs_senders.get_mut(&vstored) {
                    let _ = sender.send(sig).await;
                }
            },
            Some(Ok((vstored, quorum))) = collect_quorums_join_set.join_next() => {
                if let Some(got_quorum) = quorum {
                    let tsto = LedgeraTransaction::Tsto(
                        ProofOfShipmentToStorage::new(vstored, got_quorum)
                    );
                    {
                        let mut comm_sess = comm_api.lock().await;
                        if let Err(e) = comm_sess
                            .serialize_and_publish_on_topic::<LedgeraTransaction>(
                                comm_params,
                                &LedgeraCorePublicationTopics::TransactionSubmission
                                    .get_publication_topic_str(service.as_ref()),
                                &tsto,
                            )
                            .await
                        {
                            log::warn!(
                                "As {:?} : could not submit 'Tsto' transaction with error : {:?}",
                                LedgeraCoreRoles::VoterComputer,
                                e
                            )
                        } else {
                            log::info!(
                                "As {:?} : submitted 'Tsto' transaction",
                                LedgeraCoreRoles::VoterComputer,
                            )
                        }
                    }
                } else {
                    log::warn!(
                        "As {:?} : quorum collection channel closed before a storage quorum could be formed; skipping Tsto submission",
                        LedgeraCoreRoles::VoterComputer,
                    );
                }
                if collect_quorums_join_set.is_empty() {
                    break 'wait_until_all_quorums;
                }
            }
        }
    }

    Ok(())
}
