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

use crate::management::{
    channels::PerInstanceVoterBehaviorPhase1Receivers, error::VoterComputationBehaviorError,
};
use ledgera_comms::{
    comm_api::{LedgeraInternalCommunicationInterface, LedgeraInternalCommunicationParameters},
    comm_session::PubSubNetwork,
};
use ledgera_core_logic::{
    quorum::collect_quorum, roles::LedgeraCoreRoles, topics::LedgeraCorePublicationTopics,
};
use ledgera_pki::{
    manager::PublicKeyInfrastructure, message::SignatureEntry, quorum::QuorumOfSignatures,
};
use ledgera_types::app_template::input::LedgeraInputArgument;
use ledgera_types::app_template::spec::LedgeraAtomicOperationSpecification;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::proofs::proof_of_declaration::ProofOfFunctionDeclaration;
use ledgera_types::traits::LedgeraQuorumContainingMessage;
use ledgera_types::transactions::LedgeraTransaction;
use ledgera_types::{digest::LedgeraDigest, votes::vfun::LedgeraVoteFunctionInstanceDeclaration};

use crate::logic::outputs::ComputationInstancePhase1Result;

pub async fn phase1_access_control_logic<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
>(
    comm_api: &Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    comm_params: &Arc<LedgeraInternalCommunicationParameters<PKI>>,
    service: &Arc<LAT>,
    // ***
    // asynch receivers
    mut phase1_receivers: PerInstanceVoterBehaviorPhase1Receivers<LAT>,
) -> Result<ComputationInstancePhase1Result<LAT>, VoterComputationBehaviorError<Sess, LAT>> {
    // ========================================================================================
    // ==== Phase 1 / Part 1 :
    // ====  - wait for the initial Rfun message
    // ====  - and then emit a Vfun and start collecting f+1 Vfuns
    // ========================================================================================
    let initial_rfun_sigentry: SignatureEntry;
    let comp_spec: LedgeraAtomicOperationSpecification<LAT>;
    let vfun_vote: LedgeraVoteFunctionInstanceDeclaration;
    let mut wait_vfun_quorum: Pin<Box<dyn Future<Output = Option<QuorumOfSignatures>> + Send>>;
    tokio::select! {
        Some((sig,rfun)) = phase1_receivers.rfun_receiver.recv() => {
            // Every LP_S-backed known input must carry a valid quorum.
            if let Err(e) = rfun.verify_vote_quorums::<PKI>(
                &comm_params.known_participants,
                comm_params.byzantine_threshold,
            ) {
                log::warn!(
                    "As {:?} : received Rfun with invalid LP_S proof(s), refusing execute access: {:?}",
                    LedgeraCoreRoles::VoterComputer,
                    e
                );
                return Err(VoterComputationBehaviorError::InvalidRfun);
            }

            // TODO : also check arity match between:
            // - function arity
            // - global predicate arity
            // - arguments.len()
            // but we cannot do it yet as the LAT does not expose arities

            // TODO : predicate satisfiability check

            initial_rfun_sigentry = sig.clone();
            comp_spec = rfun.spec;
            vfun_vote = LedgeraVoteFunctionInstanceDeclaration::new(
                // function_instance_identifier
                sig.serializable_signature.clone(),
                // known_arguments
                comp_spec.arguments.iter().enumerate().filter(|(_,y)| y.is_concrete())
                .map(|(x,a)| {
                    match a {
                        LedgeraInputArgument::ReferenceToStorage(proof_of_shipment_to_storage) => {
                            (x as u32,proof_of_shipment_to_storage.v.data_digest.clone())
                        },
                        LedgeraInputArgument::RawValue{is_input_persistent : _, value : val} => {
                            let digest = LedgeraDigest::from_serializable(val).unwrap();
                            (x as u32,digest)
                        },
                        _ => {
                            unreachable!()
                        }
                    }
                }).collect(),
                // unknown_arguments_indices
                comp_spec.arguments.iter().enumerate().filter(|(_,y)| !y.is_concrete()).map(|(x,_)| x as u32).collect(),
                // persistent_inputs_indices
                comp_spec.arguments.iter().enumerate()
                .filter(|(_,y)| {
                    match y {
                        LedgeraInputArgument::RawValue { is_input_persistent, value:_ } => {
                            *is_input_persistent
                        },
                        _ => {
                            false
                        }
                    }
                })
                .map(|(x,_)| {
                    x as u32
                }).collect(),
            );

            {
                let mut comm_sess = comm_api.lock().await;
                match comm_sess
                    .serialize_and_publish_on_topic::<LedgeraVoteFunctionInstanceDeclaration>(
                        comm_params,
                        &LedgeraCorePublicationTopics::Vfun.get_publication_topic_str(service.as_ref()),
                        &vfun_vote
                    ).await {
                    Err(e) => {
                        return Err(VoterComputationBehaviorError::CouldNotEmitVfun(e));
                    },
                    Ok(()) => {
                        log::info!(
                            "As {:?} : emitted positive vote on according execute access for computation instance {:}",
                            LedgeraCoreRoles::VoterComputer,
                            sig.serializable_signature.to_hexadecimal_string()
                        )
                    }
                }
            }

            wait_vfun_quorum =
                Box::pin(
                    collect_quorum::<PKI>(
                        bincode::serialize(&vfun_vote).unwrap(),
                        phase1_receivers.vfun_receiver,
                        comm_params.byzantine_threshold as usize
                    )
                )
            ;
        }
    }

    // ========================================================================================
    // ==== Phase 1 / Part 2 :
    // ====  - wait until having collected f+1 Vfuns
    // ====  - then send a Tfun
    // ========================================================================================
    let mut pod: ProofOfFunctionDeclaration;
    tokio::select! {
        Some(got_vfun_quorum) = wait_vfun_quorum.as_mut() => {
            pod = ProofOfFunctionDeclaration::new(
                vfun_vote,
                got_vfun_quorum
            );
            let tx = LedgeraTransaction::Tfun(pod);
            {
                let mut comm_sess = comm_api.lock().await;
                match comm_sess
                    .serialize_and_publish_on_topic::<LedgeraTransaction>(
                        comm_params,
                        &LedgeraCorePublicationTopics::TransactionSubmission.get_publication_topic_str(service.as_ref()),
                        &tx
                    ).await {
                    Err(e) => {
                        return Err(VoterComputationBehaviorError::CouldNotEmitTfun(e));
                    },
                    Ok(_) => {
                        log::info!(
                            "As {:?} : submitted 'Tfun' transaction for computation instance {:}",
                            LedgeraCoreRoles::VoterComputer,
                            initial_rfun_sigentry.serializable_signature.to_hexadecimal_string()
                        )
                    }
                }
            }
            pod = match tx {
                LedgeraTransaction::Tfun(pod) => {
                    pod
                },
                _ => {
                    unreachable!()
                }
            };
        }
    }
    // finally returns the computation specification, which is needed
    Ok(ComputationInstancePhase1Result::new(
        comp_spec,
        pod,
        initial_rfun_sigentry.serialized_signing_public_key,
    ))
}
