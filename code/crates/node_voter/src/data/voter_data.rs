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


use ledgera_comms::{comm_api::{LedgeraInternalCommunicationInterface, LedgeraInternalCommunicationParameters}, comm_session::PubSubNetwork, error::LedgeraCommunicationError};
use ledgera_core_logic::{roles::LedgeraCoreRoles,  topics::LedgeraCorePublicationTopics};
use ledgera_pki::{manager::PublicKeyInfrastructure, message::SignatureEntry};
use ledgera_types::{messages::{march::LedgeraCoreRequestArch, sval::LedgeraServerSideStorageRequest, varch::LedgeraVoteArch, vstored::LedgeraVoteStored}, storage::digest::LedgeraDigest};
use std::{collections::{HashMap}, sync::Arc};

use crate::data::per_value_voter_data_state::PerValueVoterDataState;


pub struct LedgeraVoterDataState<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
> {
    comm_session: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI,Sess>>>,
    comm_params : Arc<LedgeraInternalCommunicationParameters<PKI>>,
    service : Arc<Service>,
    // ***
    react_to_input_task : Option<tokio::task::JoinHandle<()>>,
}

impl<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
    > LedgeraVoterDataState<PKI, Sess, Service>
{
    pub fn new(
        comm_session: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI,Sess>>>, 
        comm_params: Arc<LedgeraInternalCommunicationParameters<PKI>>, 
        service: Arc<Service>
    ) -> Self {
        Self { comm_session, comm_params, service, react_to_input_task:None }
    }


    pub async fn run(
        &mut self,
        mut override_receiver : tokio::sync::mpsc::Receiver<(LedgeraDigest,LedgeraServerSideStorageRequest<Service::DataValue>)>
    ) -> Result<(), LedgeraCommunicationError<Sess::CommRuntimeError>> {
        // subscribing to relevant "client requests" and "votes"
        let (march_sender, mut march_receiver) = tokio::sync::mpsc::channel(128);
        let (varch_sender, mut varch_receiver) = tokio::sync::mpsc::channel(128);
        let (vstored_sender, mut vstored_receiver) = tokio::sync::mpsc::channel(128);
        {
            let mut comm_sess = self.comm_session.lock().await;
            match comm_sess
                .subscribe_to_topic_and_deserialize_as::<LedgeraCoreRequestArch<Service::DataValue>>(
                    &self.comm_params, 
                    &self.service.get_publication_topic_str(&LedgeraCorePublicationTopics::March),
                    march_sender
                ).await
            {
                Ok(_) => {
                    log::info!(
                        "As {:?} : subscribed to March client requests for service '{:}'",
                        LedgeraCoreRoles::VoterComputer,
                        self.service.get_service_name()
                    );
                }
                Err(e) => {
                    return Err(e);
                }
            }
            match comm_sess
                .subscribe_to_topic_and_deserialize_as::<LedgeraVoteArch>(
                    &self.comm_params, 
                    &self.service.get_publication_topic_str(&LedgeraCorePublicationTopics::Varch),
                    varch_sender
                )
                .await
            {
                Ok(_) => {
                    log::info!(
                        "As {:?} : subscribed to Varch votes for service '{:}'",
                        LedgeraCoreRoles::VoterComputer,
                        self.service.get_service_name()
                    );
                }
                Err(e) => {
                    return Err(e);
                }
            }
            
        }

        let comm_session_ref = self.comm_session.clone();
        let comm_params_ref = self.comm_params.clone();
        let service_ref = self.service.clone();
        self.react_to_input_task = Some(tokio::spawn(async move {

            let mut per_value_states : HashMap<LedgeraDigest, Option<PerValueVoterDataState<Service>>> = HashMap::new();
            loop {

                enum DataMsg<DataValue> {
                    OverrideSval(LedgeraServerSideStorageRequest<DataValue>),
                    March(LedgeraCoreRequestArch<DataValue>),
                    Varch(SignatureEntry),
                    Vstored(SignatureEntry)
                }

                let mut add_new = None;
                tokio::select! {
                    Some((data_digest, sval_msg)) = override_receiver.recv() => {
                        match per_value_states.get_mut(&data_digest) {
                            Some(None) => {
                                // the archival has already been handled
                            },
                            Some(Some(per_value_state)) => {
                                let _ = per_value_state.override_sender.send(sval_msg).await;
                            },
                            None => {
                                add_new = Some((data_digest,DataMsg::OverrideSval(sval_msg)));
                            }
                        }
                    },
                    Some((march,_)) = march_receiver.recv() => {
                        let data_digest = LedgeraDigest::from_serializable(&march.value).unwrap();
                        match per_value_states.get_mut(&data_digest) {
                            Some(None) => {
                                // the archival has already been handled
                            },
                            Some(Some(per_value_state)) => {
                                let _ = per_value_state.march_sender.send(march).await;
                            },
                            None => {
                                add_new = Some((data_digest,DataMsg::March(march)));
                            }
                        }
                    },
                    Some((varch,sigentry)) = varch_receiver.recv() => {
                        match per_value_states.get_mut(&varch.data_digest) {
                            Some(None) => {
                                // the archival has already been handled
                            },
                            Some(Some(per_value_state)) => {
                                let _ = per_value_state.varch_sender.send(sigentry).await;
                            },
                            None => {
                                add_new = Some((varch.data_digest.clone(),DataMsg::Varch(sigentry)));
                            }
                        }
                    },
                    Some((vstored,sigentry)) = vstored_receiver.recv() => {
                        match per_value_states.get_mut(&vstored.data_digest) {
                            Some(None) => {
                                // the archival has already been handled
                            },
                            Some(Some(per_value_state)) => {
                                let _ = per_value_state.vstored_sender.send(sigentry).await;
                            },
                            None => {
                                add_new = Some((vstored.data_digest.clone(),DataMsg::Vstored(sigentry)));
                            }
                        }
                    }
                }

                if let Some((data_digest,got_msg)) = add_new {
                    let (per_value_march_sender, per_value_march_receiver) = tokio::sync::mpsc::channel(128);
                    let (per_value_varch_sender, per_value_varch_receiver) = tokio::sync::mpsc::channel(128);
                    let (per_value_vstored_sender, per_value_vstored_receiver) = tokio::sync::mpsc::channel(128);
                    let (per_value_override_sender, per_value_override_receiver) = tokio::sync::mpsc::channel(128);
                    match got_msg {
                        DataMsg::March(x) => {
                            let _ = per_value_march_sender.send(x).await;
                        },
                        DataMsg::Varch(x) => {
                            let _ = per_value_varch_sender.send(x).await;
                        },
                        DataMsg::Vstored(x) => {
                            let _ = per_value_vstored_sender.send(x).await;
                        },
                        DataMsg::OverrideSval(x) => {
                            let _ = per_value_override_sender.send(x).await;
                        },
                    }
                    per_value_states.insert(
                        data_digest,
                        Some(PerValueVoterDataState::<Service>::new(
                            per_value_march_sender, per_value_varch_sender, per_value_vstored_sender, per_value_override_sender
                        ))
                    );
                    let comm_session = comm_session_ref.clone();
                    let comm_params = comm_params_ref.clone();
                    let service = service_ref.clone();
                    tokio::spawn(async move {
                        PerValueVoterDataState::<Service>::run(
                            comm_session,comm_params,service,
                            per_value_march_receiver, 
                            per_value_varch_receiver, 
                            per_value_vstored_receiver,
                            per_value_override_receiver
                        ).await;
                    });
                }
            }

        }));
        
        Ok(())
    }
}
