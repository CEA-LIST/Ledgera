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
 
 
 
 use std::{future::Future, pin::Pin, sync::Arc};

use ledgera_comms::{comm_api::{LedgeraInternalCommunicationInterface, LedgeraInternalCommunicationParameters}, comm_session::PubSubNetwork};
use ledgera_core_logic::{quorum::collect_quorum, roles::LedgeraCoreRoles,  topics::LedgeraCorePublicationTopics};
use ledgera_pki::{manager::PublicKeyInfrastructure, message::SignatureEntry, quorum::QuorumOfSignatures};
use ledgera_types::{messages::{march::LedgeraCoreRequestArch, sval::{LedgeraServerSideStorageRequest, ProofOfWriteAccessToStorage}, transactions::{tarch::AnchorProofOfStorage, txs::LedgeraTransaction}, varch::LedgeraVoteArch, vstored::LedgeraVoteStored}, storage::digest::LedgeraDigest};




pub struct PerValueVoterDataState<LAT: LedgeraApplicationTemplate> {
    pub march_sender : tokio::sync::mpsc::Sender<LedgeraCoreRequestArch<Service::DataValue>>,
    pub varch_sender : tokio::sync::mpsc::Sender<SignatureEntry>,
    pub vstored_sender : tokio::sync::mpsc::Sender<SignatureEntry>,
    pub override_sender : tokio::sync::mpsc::Sender<LedgeraServerSideStorageRequest<Service::DataValue>>
}


impl<LAT: LedgeraApplicationTemplate> PerValueVoterDataState<Service> {
    pub fn new(
        march_sender: tokio::sync::mpsc::Sender<LedgeraCoreRequestArch<Service::DataValue>>, 
        varch_sender: tokio::sync::mpsc::Sender<SignatureEntry>, 
        vstored_sender: tokio::sync::mpsc::Sender<SignatureEntry>,
        override_sender : tokio::sync::mpsc::Sender<LedgeraServerSideStorageRequest<Service::DataValue>>
    ) -> Self {
        Self { march_sender, varch_sender, vstored_sender,override_sender }
    }


    pub async fn run<
        PKI: PublicKeyInfrastructure,
        Sess: PubSubNetwork
    >(
        comm_session: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI,Sess>>>,
        comm_params : Arc<LedgeraInternalCommunicationParameters<PKI>>,
        service : Arc<Service>,
        mut march_receiver : tokio::sync::mpsc::Receiver<LedgeraCoreRequestArch<Service::DataValue>>,
        varch_receiver : tokio::sync::mpsc::Receiver<SignatureEntry>,
        vstored_receiver : tokio::sync::mpsc::Receiver<SignatureEntry>,
        mut override_receiver : tokio::sync::mpsc::Receiver<LedgeraServerSideStorageRequest<Service::DataValue>>
    ) {
        // ========================================================================================
        // ==== First step : 
        // ====  - wait for the initial March message 
        // ====  - and then emit a Varch and start collecting f+1 Varchs
        // ========================================================================================
        let data_value : Service::DataValue;
        let mut data_digest : LedgeraDigest;
        let mut wait_varch_quorum: Pin<Box<dyn Future<Output = Option<QuorumOfSignatures>> + Send>>;
        tokio::select! {
            Some(sval_msg) = override_receiver.recv() => {
                data_digest = LedgeraDigest::from_serializable(&sval_msg.value).unwrap();
                return Self::run_once_write_access_granted(
                    comm_session,comm_params,service,data_digest,sval_msg,vstored_receiver
                ).await;
            },
            Some(march) = march_receiver.recv() => {
                data_value = march.value;
                data_digest = LedgeraDigest::from_serializable(&data_value).unwrap();
                let varch_vote = LedgeraVoteArch::new(data_digest.clone());

                { 
                    // TODO / PROVISORY : for now we always give write access
                    let mut comm_sess = comm_session.lock().await;
                    if let Err(e) = comm_sess
                        .serialize_and_publish_on_topic::<LedgeraVoteArch>(
                            &comm_params,
                            &service.get_publication_topic_str(&LedgeraCorePublicationTopics::Varch),
                            &varch_vote
                        ).await
                    {
                        log::warn!(
                            "As {:?} : could not emit vote on according write access with error : {:?}",
                            LedgeraCoreRoles::VoterComputer,
                            e
                        )
                    } else {
                        log::info!(
                            "As {:?} : emitted positive vote on according write access for data digest {:}",
                            LedgeraCoreRoles::VoterComputer,
                            data_digest.to_hexadecimal_string()
                        )
                    }
                }

                wait_varch_quorum =
                    Box::pin(
                        collect_quorum::<PKI>(
                            bincode::serialize(&varch_vote).unwrap(),
                            varch_receiver,
                            comm_params.byzantine_threshold as usize
                        )
                    )
                ;
            }
        }

        // ========================================================================================
        // ==== Second step : 
        // ====  - wait until having collected f+1 Varchs
        // ====  - then send a Sval and a Vstored and start collecting f+1 Vstored
        // ========================================================================================
        tokio::select! {
            Some(sval_msg) = override_receiver.recv() => {
                data_digest = LedgeraDigest::from_serializable(&sval_msg.value).unwrap();
                return Self::run_once_write_access_granted(
                    comm_session,comm_params,service,data_digest,sval_msg,vstored_receiver
                ).await;
            },
            Some(got_varch_quorum) = wait_varch_quorum.as_mut() => {
                let sval_msg = LedgeraServerSideStorageRequest::new(
                    data_value,
                    ProofOfWriteAccessToStorage::ViaQuorumOfVarch{quorum_of_varch : got_varch_quorum}
                );
                return Self::run_once_write_access_granted(
                    comm_session,comm_params,service,data_digest,sval_msg,vstored_receiver
                ).await;
            }
        }
        
    }





}








