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

use futures::FutureExt;
use ledgera_comms::{
    comm_api::{LedgeraInternalCommunicationInterface, LedgeraInternalCommunicationParameters},
    comm_session::PubSubNetwork,
    error::LedgeraCommunicationError,
};
use ledgera_core_logic::roles::LedgeraCoreRoles;
use ledgera_pki::manager::{PublicKeyInfrastructure, SerdeSerializable64BitsSignature};
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::transactions::LedgeraTransaction;
use ledgera_types::votes::vsto::PersistentDataKind;
use std::{collections::HashMap, sync::Arc};
use tokio::task::JoinSet;

use crate::{
    logic::instance_logic::run_computation_instance_logic,
    management::{
        channels::{
            initiate_computation_instance_state_on_voter, InitialMessageCreatingInstanceState,
            PerInstanceVoterBehaviorSenders,
        },
        error::VoterComputationBehaviorError,
        subscription::{make_subscriptions_for_voter_node, SendersForVoterNodeSubscriptions},
    },
};

pub struct LedgeraVoterBehavior<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
> {
    comm_session: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    comm_params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
    service: Arc<LAT>,
    // ***
    react_to_input_task: Option<tokio::task::JoinHandle<()>>,
}

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork, LAT: LedgeraApplicationTemplate>
    LedgeraVoterBehavior<PKI, Sess, LAT>
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
            react_to_input_task: None,
        }
    }

    pub async fn run(&mut self) -> Result<(), LedgeraCommunicationError<Sess::CommRuntimeError>> {
        // subscribing to relevant "client requests", "votes" and "delivered transactions"
        let (rfun_sender, mut rfun_receiver) = tokio::sync::mpsc::channel(128);
        let (rin_sender, mut rin_receiver) = tokio::sync::mpsc::channel(128);
        let (vfun_sender, mut vfun_receiver) = tokio::sync::mpsc::channel(128);
        let (vins_sender, mut vins_receiver) = tokio::sync::mpsc::channel(128);
        let (vout_sender, mut vout_receiver) = tokio::sync::mpsc::channel(128);
        let (delivered_txs_sender, mut delivered_txs_receiver) = tokio::sync::mpsc::channel(128);
        let (vstored_sender, mut vstored_receiver) = tokio::sync::mpsc::channel(128);
        {
            make_subscriptions_for_voter_node(
                &self.comm_session,
                &self.comm_params,
                &self.service,
                // ***
                SendersForVoterNodeSubscriptions {
                    rfun_sender,
                    rin_sender,
                    vfun_sender,
                    vins_sender,
                    vout_sender,
                    delivered_txs_sender,
                    vstored_sender,
                },
            )
            .await?;
        }

        let comm_session_ref = self.comm_session.clone();
        let comm_params_ref = self.comm_params.clone();
        let service_ref = self.service.clone();
        self.react_to_input_task = Some(tokio::spawn(async move {
            // keeps track of all voter states, for all instances of computation.
            // KNOWN LIMITATION: tombstone entries (None values) are never removed.
            // When a computation instance terminates, its fid is kept mapped to None so that
            // late-arriving stale messages for that instance are silently ignored rather than
            // re-opening a new instance. This mirrors the spec's own `terminated` set (Alg. 10),
            // which also grows without bound. Over very long-lived nodes that process many
            // function instances, this map will grow monotonically and eventually apply memory
            // pressure. The correct fix requires a garbage-collection policy — e.g., evicting
            // tombstones after the secure log has delivered D(T_out) for the corresponding fid,
            // guaranteeing no further messages for that instance will be considered valid.
            let mut per_computation_instance_states: HashMap<
                SerdeSerializable64BitsSignature,
                Option<PerInstanceVoterBehaviorSenders<LAT>>,
            > = HashMap::new();

            struct FunctionInstanceLogicThreadArtifact<
                Sess: PubSubNetwork,
                LAT: LedgeraApplicationTemplate,
            > {
                pub fid: SerdeSerializable64BitsSignature,
                pub fi_thread_retval: Result<(), VoterComputationBehaviorError<Sess, LAT>>,
            }

            let mut instance_logic_threads_join_set: JoinSet<
                FunctionInstanceLogicThreadArtifact<Sess, LAT>,
            > = JoinSet::new();
            loop {
                // wait for the next message (of any kind in {Rfun,Vfun,Rin,Vins,Vout,deliveredTransaction}) and:
                // - if there is not existing computation instance state for that message, create a new one and forward the message to it
                // - if there already exist an active relevant computation instance state then try forwarding the message to it and:
                //   + if the message can be forwarded do nothing
                //   + otherwise it means that the computation instance is completed and we may garbade collect the "PerInstanceVoterComputationStateSenders"
                // - if the corresponding computation instance state has already terminate, this is a stale message and we do nothing
                enum OnReceiveComputationInstanceMessageAction<LAT: LedgeraApplicationTemplate> {
                    CreateInstance(
                        SerdeSerializable64BitsSignature,
                        Box<InitialMessageCreatingInstanceState<LAT>>,
                    ),
                    DeleteInstance(SerdeSerializable64BitsSignature),
                    Nothing,
                }

                let action;
                tokio::select! {
                    Some(join_result) = instance_logic_threads_join_set.join_next() => {
                        match join_result {
                            Ok(fi_thread_artifact) => {
                                match fi_thread_artifact.fi_thread_retval {
                                    Ok(_) => {
                                        log::info!(
                                            "As {:?} : computation instance thread has terminated correctly for instance '{:}'",
                                            LedgeraCoreRoles::VoterComputer,
                                            fi_thread_artifact.fid.to_hexadecimal_string()
                                        );
                                    },
                                    Err(e) => {
                                        log::warn!(
                                            "As {:?} : computation instance thread has terminated with error '{:?}' for instance '{:}'",
                                            LedgeraCoreRoles::VoterComputer,
                                            e,
                                            fi_thread_artifact.fid.to_hexadecimal_string()
                                        );
                                    }
                                }
                                action = OnReceiveComputationInstanceMessageAction::DeleteInstance(fi_thread_artifact.fid);
                            },
                            Err(join_error) => {
                                // Task was cancelled — fid is not recoverable from JoinError.
                                // This should not occur since no code calls abort() on these tasks.
                                log::warn!(
                                    "As {:?} : a computation instance task was unexpectedly cancelled: {:?}",
                                    LedgeraCoreRoles::VoterComputer,
                                    join_error
                                );
                                action = OnReceiveComputationInstanceMessageAction::Nothing;
                            }
                        }
                    },
                    Some((rfun,sig)) = rfun_receiver.recv() => {
                        log::info!(
                            "As {:?} : received Rfun for computation instance '{:}'",
                            LedgeraCoreRoles::VoterComputer,
                            sig.serializable_signature.to_hexadecimal_string()
                        );
                        match per_computation_instance_states.get_mut(&sig.serializable_signature) {
                            Some(None) => {
                                action = OnReceiveComputationInstanceMessageAction::Nothing;
                            },
                            Some(Some(per_instance_state)) => {
                                let _ = per_instance_state.rfun_sender.send((sig,rfun)).await;
                                action = OnReceiveComputationInstanceMessageAction::Nothing;
                            },
                            None => {
                                action = OnReceiveComputationInstanceMessageAction::CreateInstance(
                                    sig.serializable_signature.clone(),
                                    Box::new(InitialMessageCreatingInstanceState::<LAT>::Rfun(sig,rfun))
                                );
                            }
                        }
                    },
                    Some((vfun,sigentry)) = vfun_receiver.recv() => {
                        log::info!(
                            "As {:?} : received Vfun for computation instance '{:}'",
                            LedgeraCoreRoles::VoterComputer,
                            vfun.function_instance_identifier.to_hexadecimal_string()
                        );
                        match per_computation_instance_states.get_mut(&vfun.function_instance_identifier) {
                            Some(None) => {
                                action = OnReceiveComputationInstanceMessageAction::Nothing;
                            },
                            Some(Some(per_instance_state)) => {
                                let _ = per_instance_state.vfun_sender.send(sigentry).await;
                                action = OnReceiveComputationInstanceMessageAction::Nothing;
                            },
                            None => {
                                action = OnReceiveComputationInstanceMessageAction::CreateInstance(
                                    vfun.function_instance_identifier,
                                    Box::new(InitialMessageCreatingInstanceState::Vfun(sigentry))
                                );
                            }
                        }
                    },
                    Some((rin,sigentry)) = rin_receiver.recv() => {
                        match per_computation_instance_states.get_mut(&rin.function_instance_identifier) {
                            Some(None) => {
                                // the computation instance has already terminated
                                // so we do nothing with the argument proposal
                                // which has been received too late to be even considered
                                log::info!(
                                    "As {:?} : received LATE Rin (which will be ignored) for computation instance '{:}'",
                                    LedgeraCoreRoles::VoterComputer,
                                    rin.function_instance_identifier.to_hexadecimal_string()
                                );
                                action = OnReceiveComputationInstanceMessageAction::Nothing;
                            },
                            Some(Some(per_instance_state)) => {
                                log::info!(
                                    "As {:?} : received ON_TIME Rin for computation instance '{:}'",
                                    LedgeraCoreRoles::VoterComputer,
                                    rin.function_instance_identifier.to_hexadecimal_string()
                                );
                                let _ = per_instance_state.rin_sender.send((sigentry,rin)).await;
                                action = OnReceiveComputationInstanceMessageAction::Nothing;
                            },
                            None => {
                                log::info!(
                                    "As {:?} : received EARLY Rin (has not yet received Rfun) for computation instance '{:}'",
                                    LedgeraCoreRoles::VoterComputer,
                                    rin.function_instance_identifier.to_hexadecimal_string()
                                );
                                action = OnReceiveComputationInstanceMessageAction::CreateInstance(
                                    rin.function_instance_identifier.clone(),
                                    Box::new(InitialMessageCreatingInstanceState::Rin(sigentry,rin))
                                );
                            }
                        }
                    }
                    Some((vins,sigentry)) = vins_receiver.recv() => {
                        log::info!(
                            "As {:?} : received Vins for computation instance '{:}'",
                            LedgeraCoreRoles::VoterComputer,
                            vins.function_instance_identifier.to_hexadecimal_string()
                        );
                        match per_computation_instance_states.get_mut(&vins.function_instance_identifier) {
                            Some(None) => {
                                action = OnReceiveComputationInstanceMessageAction::Nothing;
                            },
                            Some(Some(per_instance_state)) => {
                                let _ = per_instance_state.vins_sender.send((sigentry,vins)).await;
                                action = OnReceiveComputationInstanceMessageAction::Nothing;
                            },
                            None => {
                                action = OnReceiveComputationInstanceMessageAction::CreateInstance(
                                    vins.function_instance_identifier.clone(),
                                    Box::new(InitialMessageCreatingInstanceState::Vins(sigentry,vins))
                                );
                            }
                        }
                    },
                    Some((vout,sigentry)) = vout_receiver.recv() => {
                        log::info!(
                            "As {:?} : received Vout for computation instance '{:}'",
                            LedgeraCoreRoles::VoterComputer,
                            vout.function_instance_identifier.to_hexadecimal_string()
                        );
                        match per_computation_instance_states.get_mut(&vout.function_instance_identifier) {
                            Some(None) => {
                                action = OnReceiveComputationInstanceMessageAction::Nothing;
                            },
                            Some(Some(per_instance_state)) => {
                                let _ = per_instance_state.vout_sender.send((sigentry,vout)).await;
                                action = OnReceiveComputationInstanceMessageAction::Nothing;
                            },
                            None => {
                                action = OnReceiveComputationInstanceMessageAction::CreateInstance(
                                    vout.function_instance_identifier.clone(),
                                    Box::new(InitialMessageCreatingInstanceState::Vout(sigentry,vout))
                                );
                            }
                        }
                    },
                    Some((vstored,sigentry)) = vstored_receiver.recv() => {
                        log::info!(
                            "As {:?} : received Vstored for computation instance '{:}'",
                            LedgeraCoreRoles::VoterComputer,
                            vstored.function_instance_identifier.to_hexadecimal_string()
                        );
                        match per_computation_instance_states.get_mut(&vstored.function_instance_identifier) {
                            Some(None) => {
                                action = OnReceiveComputationInstanceMessageAction::Nothing;
                            },
                            Some(Some(per_instance_state)) => {
                                if matches!(vstored.data_kind, PersistentDataKind::Input(_)) {
                                    let _ = per_instance_state.inputs_vstored_sender.send((sigentry,vstored)).await;
                                } else {
                                    let _ = per_instance_state.output_vstored_sender.send((sigentry,vstored)).await;
                                }
                                action = OnReceiveComputationInstanceMessageAction::Nothing;
                            },
                            None => {
                                let fid = vstored.function_instance_identifier.clone();
                                let initial = if matches!(vstored.data_kind, PersistentDataKind::Input(_)) {
                                    InitialMessageCreatingInstanceState::VstoInput(sigentry, vstored)
                                } else {
                                    InitialMessageCreatingInstanceState::VstoOutput(sigentry, vstored)
                                };
                                action = OnReceiveComputationInstanceMessageAction::CreateInstance(
                                    fid,
                                    Box::new(initial)
                                );
                            }
                        }
                    },
                    Some((delivered_tx,_)) = delivered_txs_receiver.recv() => {
                        let got_tx = match delivered_tx.transaction {
                            LedgeraTransaction::Tsto(_) => {
                                None
                            },
                            LedgeraTransaction::Tfun(x) => {
                                log::info!(
                                    "As {:?} : received delivered Tfun notification for computation instance '{:}'",
                                    LedgeraCoreRoles::VoterComputer,
                                    x.v.function_instance_identifier.to_hexadecimal_string()
                                );
                                Some(
                                    (x.v.function_instance_identifier.clone(),InitialMessageCreatingInstanceState::Tfun(x))
                                )
                            },
                            LedgeraTransaction::Tins(x) => {
                                log::info!(
                                    "As {:?} : received delivered Tins notification for computation instance '{:}'",
                                    LedgeraCoreRoles::VoterComputer,
                                    x.v.function_instance_identifier.to_hexadecimal_string()
                                );
                                Some(
                                    (x.v.function_instance_identifier.clone(),InitialMessageCreatingInstanceState::Tins(x))
                                )
                            },
                            LedgeraTransaction::Tout(x) => {
                                log::info!(
                                    "As {:?} : received delivered Tout notification for computation instance '{:}'",
                                    LedgeraCoreRoles::VoterComputer,
                                    x.v.function_instance_identifier.to_hexadecimal_string()
                                );
                                Some(
                                    (x.v.function_instance_identifier.clone(),InitialMessageCreatingInstanceState::Tout(x))
                                )
                            },
                        };
                        if let Some((function_instance_identifier,x)) = got_tx {
                            match per_computation_instance_states.get_mut(&function_instance_identifier) {
                                Some(None) => {
                                    action = OnReceiveComputationInstanceMessageAction::Nothing;
                                },
                                Some(Some(per_instance_state)) => {
                                    match x {
                                        InitialMessageCreatingInstanceState::Tfun(anchor_computation_instance_declaration) => {
                                            let _ = per_instance_state.tcomp_sender.send(anchor_computation_instance_declaration).await;
                                            action = OnReceiveComputationInstanceMessageAction::Nothing;
                                        },
                                        InitialMessageCreatingInstanceState::Tins(anchor_agreement_on_unknown_input_arguments) => {
                                            let _ = per_instance_state.tins_sender.send(anchor_agreement_on_unknown_input_arguments).await;
                                            action = OnReceiveComputationInstanceMessageAction::Nothing;
                                        },
                                        InitialMessageCreatingInstanceState::Tout(anchor_proof_of_integrity) => {
                                            let _ = per_instance_state.tout_sender.send(anchor_proof_of_integrity).await;
                                            action = OnReceiveComputationInstanceMessageAction::Nothing;
                                        },
                                        _ => {unreachable!()}
                                    }
                                },
                                None => {
                                    action = OnReceiveComputationInstanceMessageAction::CreateInstance(
                                        function_instance_identifier,
                                        Box::new(x)
                                    );
                                }
                            }
                        } else {
                            // do not reach to a Tarch delivered transaction
                            action = OnReceiveComputationInstanceMessageAction::Nothing;
                        }
                    },
                }

                match action {
                    OnReceiveComputationInstanceMessageAction::CreateInstance(fid, got_msg) => {
                        let (senders, receivers) =
                            initiate_computation_instance_state_on_voter(*got_msg).await;
                        per_computation_instance_states.insert(fid.clone(), Some(senders));
                        let comm_session = comm_session_ref.clone();
                        let comm_params = comm_params_ref.clone();
                        let service = service_ref.clone();
                        instance_logic_threads_join_set.spawn(async move {
                            let fi_thread_retval =
                                std::panic::AssertUnwindSafe(run_computation_instance_logic(
                                    fid.to_hexadecimal_string(),
                                    comm_session,
                                    comm_params,
                                    service,
                                    receivers,
                                ))
                                .catch_unwind()
                                .await;
                            match fi_thread_retval {
                                Ok(r) => FunctionInstanceLogicThreadArtifact {
                                    fid,
                                    fi_thread_retval: r,
                                },
                                Err(_) => FunctionInstanceLogicThreadArtifact {
                                    fid,
                                    fi_thread_retval: Err(VoterComputationBehaviorError::Panicked),
                                },
                            }
                        });
                    }
                    OnReceiveComputationInstanceMessageAction::DeleteInstance(comp_instance_id) => {
                        // Insert a tombstone (None) so that stale messages arriving after
                        // termination are ignored rather than spawning a new instance.
                        // See KNOWN LIMITATION on per_computation_instance_states above.
                        per_computation_instance_states.insert(comp_instance_id, None);
                    }
                    OnReceiveComputationInstanceMessageAction::Nothing => {
                        // do nothing as there is nothing to do
                    }
                }
            }
        }));

        Ok(())
    }
}
