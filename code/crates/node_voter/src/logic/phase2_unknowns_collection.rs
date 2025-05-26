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

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::logic::outputs::ComputationInstancePhase1Result;
use crate::logic::outputs::ComputationInstancePhase2Result;
use crate::management::channels::PerInstanceVoterBehaviorPhase2Receivers;
use crate::{
    logic::phase2::{
        placement::ArgumentsPlacementFinder,
        utils::{
            is_rin_locally_valid, is_rin_unknown_input_proposal_initially_valid,
            retrieve_known_inputs,
        },
    },
    management::error::VoterComputationBehaviorError,
};
use ledgera_comms::{
    comm_api::{LedgeraInternalCommunicationInterface, LedgeraInternalCommunicationParameters},
    comm_session::PubSubNetwork,
};
use ledgera_core_logic::{
    queries::query_data::retrieve_data_from_storage, quorum::collect_quorum,
    roles::LedgeraCoreRoles, topics::LedgeraCorePublicationTopics,
};
use ledgera_pki::{manager::PublicKeyInfrastructure, message::SignatureEntry};
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::digest::LedgeraDigest;
use ledgera_types::requests::rin::LedgeraRequestInputProposal;
use ledgera_types::transactions::LedgeraTransaction;
use ledgera_types::{
    proofs::proof_of_unknown_arguments_assignment_verification::ProofOfUnknownArgumentsAssignmentVerification,
    votes::vins::{LedgeraVoteIns, LedgeraVoteInsInputProposalReference},
};
use tokio::task::JoinSet;

pub async fn phase2_unknowns_collection_logic<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
>(
    comm_api: &Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    comm_params: &Arc<LedgeraInternalCommunicationParameters<PKI>>,
    service: &Arc<LAT>,
    // ***
    // outputs from previous phases
    phase1_result: &ComputationInstancePhase1Result<LAT>,
    // ***
    // asynch receivers
    mut receivers: PerInstanceVoterBehaviorPhase2Receivers,
) -> Result<ComputationInstancePhase2Result<LAT>, VoterComputationBehaviorError<Sess, LAT>> {
    if phase1_result.pod.v.unknown_arguments_indices.is_empty() {
        // if there are no unknown arguments in the operation specification
        if phase1_result.op_spec.operation.is_tag_inputs() {
            // if the operation is a Tag, nothing to do
            return Ok(ComputationInstancePhase2Result::new(
                HashMap::new(),
                None,
                None,
            ));
        } else {
            // if the operation is a concrete computation,
            // we need to retrive the raw
            // values of its inputs
            let known_arguments_values = retrieve_known_inputs(
                comm_api,
                comm_params,
                service,
                &phase1_result.op_spec.arguments,
            )
            .await;
            return Ok(ComputationInstancePhase2Result::new(
                known_arguments_values,
                None,
                None,
            ));
        }
    }

    // if we reach here, this means there are unknowns in the operation specification
    // and we need to perform the core-set algorithm to resolve these unknowns

    // if the operation is non-trivial or if there is a global predicate to evaluate,
    // we need to retrieve the raw values of the known inputs
    let known_arguments_values = if (!phase1_result.op_spec.operation.is_tag_inputs())
        || phase1_result.op_spec.global_arguments_predicate.is_some()
    {
        retrieve_known_inputs(
            comm_api,
            comm_params,
            service,
            &phase1_result.op_spec.arguments,
        )
        .await
    } else {
        HashMap::new()
    };

    // ===========================================================
    // ==== Management of "Rin" argument proposals ==============
    // ===========================================================
    // keeps track of received argument proposals to avoid treating duplicates
    let mut previously_received_rin_sigs: HashSet<SignatureEntry> = HashSet::new();
    // tasks to check the local validity of an argument proposal
    let mut check_local_argument_proposal_join_set = JoinSet::new();
    // keeps track of arguments proposals that failed local validation
    let mut locally_invalid_argument_proposals: HashSet<SignatureEntry> = HashSet::new();

    // ===========================================================
    // ==== Management of concrete data retrieved from storage ===
    // ===========================================================
    // tasks to retrieve from storage concrete data referred to in argument proposals
    let mut retrieve_data_join_set = JoinSet::new();
    // cache data to avoid repeating storage requests
    let mut cached_retrieved_data: HashMap<LedgeraDigest, LAT::Data> = HashMap::new();

    // ===========================================================
    // ==== Placement finder =====================================
    // ===========================================================
    let mut placement_finder = ArgumentsPlacementFinder::<LAT>::new(
        phase1_result.op_spec.arguments.len() as u32,
        &phase1_result.pod.v.unknown_arguments_indices,
    );

    // ===========================================================
    // ==== Emission & Echo of 'Vins' votes =====================
    // ===========================================================
    // wether or not this given node has already emitted its single
    // authorized spontaneous "Vins"
    let mut has_emitted_spontaneous_vins = false;
    // keeps track of digests of "Vins" that are invalid
    let mut invalid_vinss: HashSet<LedgeraDigest> = HashSet::new();
    // keeps track of the digests of "Vins" the node has already echoed
    let mut echoed_vinss: HashMap<LedgeraDigest, LedgeraVoteIns> = HashMap::new();
    // tasks to echo received 'Vins' votes
    let mut echo_vins_join_set = JoinSet::new();
    // 'Vins's we have received but do not yet know we should echo or not
    let mut pending_vinss_to_echo: HashSet<(LedgeraDigest, SignatureEntry, LedgeraVoteIns)> =
        HashSet::new();

    // ===========================================================
    // ==== 'Vins' Quorum collection ============================
    // ===========================================================
    // senders to send signatures of distinct 'Vins' votes, identified by the 'Vins' digest
    let mut vins_vote_sigs_senders: HashMap<
        LedgeraDigest,
        tokio::sync::mpsc::Sender<SignatureEntry>,
    > = HashMap::new();
    // tasks to collect quorums on "Vins" vote on different potential arguments mappings
    let mut collect_quorums_join_set = JoinSet::new();

    // ===============================================================
    // ==== once the first loop is over we get an object of this type
    // ===============================================================
    enum CollectedAllArguments {
        ViaMintingATins(ProofOfUnknownArgumentsAssignmentVerification),
        ViaReceivingADeliveredTins(ProofOfUnknownArgumentsAssignmentVerification),
    }

    let collected_arguments: CollectedAllArguments;

    'wait_events: loop {
        tokio::select! {
            // received concrete value from storage
            Some(Ok((rin_sig_entry,rin,data))) = retrieve_data_join_set.join_next() => {
                let data_digest = LedgeraDigest::from_serializable(&data).unwrap();
                // we cache the retrieved data
                cached_retrieved_data.insert(data_digest,data);
                // and we trigger the verification of the local predicate of the corresponding "Rin"
                check_local_argument_proposal_join_set.spawn(async move {
                    (rin_sig_entry,rin)
                });
            },
            // receive a Rin message
            Some((rin_sig_entry,rin)) = receivers.rin_receiver.recv() => {
                if locally_invalid_argument_proposals.contains(&rin_sig_entry) {
                    log::info!(
                        "As {:?} : received duplicate perviously flagged invalid Rin .. ignoring it",
                        LedgeraCoreRoles::VoterComputer
                    );
                    continue 'wait_events;
                }
                if !is_rin_unknown_input_proposal_initially_valid::<PKI>(
                    &phase1_result.pod.v.unknown_arguments_indices,
                    &rin,
                    &comm_params.known_participants,
                    comm_params.byzantine_threshold,
                ) {
                    locally_invalid_argument_proposals.insert(rin_sig_entry);
                    continue 'wait_events;
                }
                // we ignore duplicated Rin
                if previously_received_rin_sigs.contains(&rin_sig_entry) {
                    log::info!(
                        "As {:?} : received duplicate Rin .. ignoring it",
                        LedgeraCoreRoles::VoterComputer
                    );
                    continue 'wait_events;
                }
                previously_received_rin_sigs.insert(rin_sig_entry.clone());
                // given this is a new initially valid "Rin" proposal, we have to retrieve the concrete value
                // and check that it satisfies the predicates of the correponding argument indices in the operation's specification
                if cached_retrieved_data.contains_key(&rin.input_data.v.data_digest) {
                    // if we have already cached this value
                    // we directly trigger the verification of the local predicate
                    check_local_argument_proposal_join_set.spawn(async move {
                        (rin_sig_entry,rin)
                    });
                } else {
                    // otherwise we request the data from storage
                    let comm_api_clone = comm_api.clone();
                    let comm_params_clone = comm_params.clone();
                    let service_clone = service.clone();
                    retrieve_data_join_set.spawn(async move {
                        let store_ref = rin.input_data.clone();
                        (rin_sig_entry,rin,retrieve_data_from_storage(
                            &service_clone,
                            comm_api_clone,
                            comm_params_clone,
                            &store_ref
                        ).await)
                    });
                }
            },
            // it is time to check the local predicates on a single argument proposal
            Some(Ok((rin_sig_entry,rin))) = check_local_argument_proposal_join_set.join_next() => {
                let rin_referred_data = cached_retrieved_data.get(&rin.input_data.v.data_digest).unwrap();
                // we check that the argument passes all the predicates, for the indices at which it may be placed
                if is_rin_locally_valid::<LAT>(
                    &phase1_result.pod.v.function_instance_identifier,
                    &phase1_result.op_spec,
                    &rin,
                    rin_referred_data
                ) {
                    placement_finder.acknowledge_new_locally_valid_rin(
                        rin_sig_entry,
                        rin
                    );
                    // given that we have a new available valid argument
                    // we might get a full mapping for all unkwnows
                    // so if we have not already emitted our single spontaneous "Vins", we try to do so
                    if !has_emitted_spontaneous_vins {
                        if let Some(proposed_unknowns_assignment) = placement_finder.try_find_unknowns_assignment(
                            &phase1_result.op_spec.global_arguments_predicate,
                            &known_arguments_values,
                            &cached_retrieved_data
                        ) {
                            let vins_vote = LedgeraVoteIns::new(
                                phase1_result.pod.v.function_instance_identifier.clone(),
                                proposed_unknowns_assignment
                            );
                            {
                                let mut comm_sess = comm_api.lock().await;
                                match comm_sess
                                    .serialize_and_publish_on_topic::<LedgeraVoteIns>(
                                        comm_params,
                                        &LedgeraCorePublicationTopics::Vins.get_publication_topic_str(service.as_ref()),
                                        &vins_vote
                                    ).await {
                                    Err(e) => {
                                        return Err(VoterComputationBehaviorError::CouldNotEmitVins(e));
                                    },
                                    Ok(()) => {
                                        log::info!(
                                            "As {:?} : emitted sponteneous Vins vote for computation instance {:}",
                                            LedgeraCoreRoles::VoterComputer,
                                            phase1_result.pod.v.function_instance_identifier.to_hexadecimal_string()
                                        )
                                    }
                                }
                            }
                            has_emitted_spontaneous_vins = true;
                            let vins_digest = LedgeraDigest::from_serializable(&vins_vote).unwrap();
                            echoed_vinss.insert(
                                vins_digest.clone(),vins_vote.clone()
                            );
                            {
                                let (vins_sig_sender,vins_sig_receiver) = tokio::sync::mpsc::channel(128);
                                vins_vote_sigs_senders.insert(
                                    vins_digest.clone(),
                                    vins_sig_sender
                                );
                                let byzantine_threshold = comm_params.byzantine_threshold as usize;
                                collect_quorums_join_set.spawn(
                                    async move {
                                        let quorum = collect_quorum::<PKI>(
                                            bincode::serialize(&vins_vote).unwrap(),
                                            vins_sig_receiver,
                                            byzantine_threshold
                                        ).await;
                                        (vins_vote,quorum)
                                    }
                                );
                            }
                        }
                    }
                } else {
                    locally_invalid_argument_proposals.insert(rin_sig_entry);
                }
                // we check if considering this new 'Rin' has changed the evaluation of
                // previously received 'Vins's that are pending to be echoed
                let mut pending_vinss_to_echo2 : HashSet<(LedgeraDigest,SignatureEntry,LedgeraVoteIns)> = HashSet::new();
                for (pending_vins_digest,pending_vins_sig,pending_vins) in pending_vinss_to_echo {
                    // if any of the referred 'Rin' is invalid, we ignore the 'Vins'
                    if !pending_vins.proposed_unknowns_assignment.values().any(
                        |vins_arg_ref| {
                            locally_invalid_argument_proposals.contains(&vins_arg_ref.signature_of_rin)
                    }) {
                        if pending_vins.proposed_unknowns_assignment.values().all(
                            |vins_arg_ref| {
                                placement_finder.get_locally_validated_argument_proposals().contains_key(&vins_arg_ref.signature_of_rin)
                            }
                        ) {
                            echo_vins_join_set.spawn(async move {
                                (pending_vins_digest,pending_vins_sig,pending_vins)
                            });
                        } else {
                            pending_vinss_to_echo2.insert((pending_vins_digest,pending_vins_sig,pending_vins));
                        }
                    }
                }
                pending_vinss_to_echo = pending_vinss_to_echo2;
            }
            // check "Vins" then echo it if ok
            Some(Ok((pending_vins_digest,pending_vins_sig,pending_vins))) = echo_vins_join_set.join_next() => {
                if invalid_vinss.contains(&pending_vins_digest) {
                    continue 'wait_events;
                }
                if let Some(vins_sig_sender) = vins_vote_sigs_senders.get_mut(&pending_vins_digest) {
                    // forward signature to the collect_quorum task in the joinset
                    let _ = vins_sig_sender.send(pending_vins_sig).await;
                } else {
                    // we need to check that the "Vins" refers to a correct mapping on arguments
                    if placement_finder.verify_vins_proposed_unknowns_assignment_validity(
                        &pending_vins.proposed_unknowns_assignment,
                        &phase1_result.op_spec.global_arguments_predicate,
                        &known_arguments_values,
                        &cached_retrieved_data
                    ) {
                        // we echo the vote
                        {
                            let mut comm_sess = comm_api.lock().await;
                            match comm_sess
                                .serialize_and_publish_on_topic::<LedgeraVoteIns>(
                                    comm_params,
                                    &LedgeraCorePublicationTopics::Vins.get_publication_topic_str(service.as_ref()),
                                    &pending_vins
                                ).await {
                                Err(e) => {
                                    return Err(VoterComputationBehaviorError::CouldNotEmitVins(e));
                                },
                                Ok(()) => {
                                    log::info!(
                                        "As {:?} : echoed Vins vote received from another node for computation instance {:}",
                                        LedgeraCoreRoles::VoterComputer,
                                        phase1_result.pod.v.function_instance_identifier.to_hexadecimal_string()
                                    )
                                }
                            }
                        }
                        echoed_vinss.insert(
                            pending_vins_digest.clone(),pending_vins.clone()
                        );
                        // we trigger the collection of the "Vins" quorum
                        {
                            let (vins_sig_sender,vins_sig_receiver) = tokio::sync::mpsc::channel(128);
                            vins_vote_sigs_senders.insert(
                                pending_vins_digest,
                                vins_sig_sender
                            );
                            let byzantine_threshold = comm_params.byzantine_threshold as usize;
                            collect_quorums_join_set.spawn(
                                async move {
                                    let quorum = collect_quorum::<PKI>(
                                        bincode::serialize(&pending_vins).unwrap(),
                                        vins_sig_receiver,
                                        byzantine_threshold
                                    ).await;
                                    (pending_vins,quorum)
                                }
                            );
                        }
                    }
                }
            },
            // receive a Vins vote
            Some((vins_sig,vins)) = receivers.vins_receiver.recv() => {
                let vins_digest = LedgeraDigest::from_serializable(&vins).unwrap();
                if invalid_vinss.contains(&vins_digest) {
                    continue 'wait_events;
                }
                match vins_vote_sigs_senders.get_mut(&vins_digest) {
                    Some(vins_sig_sender) => {
                        // forward signature to the collect_quorum task in the joinset
                        let _ = vins_sig_sender.send(vins_sig).await;
                    },
                    None => {
                        // if the 'Vins' is not well-formed we ignore it
                        if let Err(e) = vins.verify_traceability_of_each_input_to_real_rin::<PKI>(
                            &comm_params.known_participants,
                            comm_params.byzantine_threshold,
                        ) {
                            log::info!(
                                "As {:?} : received a malformed 'Vins' vote : {:?}",
                                LedgeraCoreRoles::VoterComputer,
                                e
                            );
                            invalid_vinss.insert(vins_digest);
                            continue 'wait_events;
                        }
                        // if any of the referred 'Rin' is invalid, we ignore the 'Vins'
                        if vins.proposed_unknowns_assignment.values().any(
                            |vins_arg_ref| {
                                locally_invalid_argument_proposals.contains(&vins_arg_ref.signature_of_rin)
                        }) {
                            invalid_vinss.insert(vins_digest);
                            continue 'wait_events;
                        }
                        // we see which referred 'Rin' we have not yet locally validated
                        let missing_args : HashMap<&u32,&LedgeraVoteInsInputProposalReference> = vins.proposed_unknowns_assignment.iter().filter(
                            |(_,vins_arg_ref)| {
                                !placement_finder.get_locally_validated_argument_proposals().contains_key(&vins_arg_ref.signature_of_rin)
                            }
                        ).collect();
                        // if we have locally validated all referedd 'Rin', we can now echo the 'Vins'
                        if missing_args.is_empty() {
                            echo_vins_join_set.spawn(async move {
                                (vins_digest,vins_sig,vins)
                            });
                        } else {
                            // otherwise for any of the 'Rin' which are referred,
                            // we do as if we receive them directly from the network
                            for (_,vins_arg_ref) in missing_args {
                                let rin = LedgeraRequestInputProposal::new(
                                    phase1_result.pod.v.function_instance_identifier.clone(),
                                    vins_arg_ref.argument_indices.clone(),
                                    vins_arg_ref.pos.clone()
                                );
                                if let Err(e) = receivers.rin_sender_clone.send(
                                    (vins_arg_ref.signature_of_rin.clone(),rin)
                                ).await {
                                    log::info!(
                                        "As {:?} : internal error in core-set algorithm due to tokio mpsc channel : {:}",
                                        LedgeraCoreRoles::VoterComputer,
                                        e
                                    );
                                };
                            }
                            pending_vinss_to_echo.insert(
                                (vins_digest,vins_sig,vins)
                            );
                        }
                    },
                }
            },
            // collected a quorum of "Vins" votes
            Some(Ok((vins_vote,quorum))) = collect_quorums_join_set.join_next() => {
                if let Some(got_quorum) = quorum {
                    let pouav = ProofOfUnknownArgumentsAssignmentVerification::new(
                        vins_vote,
                        got_quorum
                    );
                    collected_arguments = CollectedAllArguments::ViaMintingATins(pouav);
                    break 'wait_events;
                }
            },
            // we receive a delivered "Tins" transaction
            Some(tins) = receivers.tins_receiver.recv() => {
                // this means some other node has submitted a valid "Tins" and it has been ordered by the orderers
                // so no need for this node to keep collecting arguments, votes etc
                collected_arguments = CollectedAllArguments::ViaReceivingADeliveredTins(tins);
                break 'wait_events;
            }
        }
    }

    let final_pouav = match collected_arguments {
        CollectedAllArguments::ViaMintingATins(pouav) => {
            // we create and submit a new "Tins" transaction
            let tins = LedgeraTransaction::Tins(pouav);
            {
                let mut comm_sess = comm_api.lock().await;
                match comm_sess
                    .serialize_and_publish_on_topic::<LedgeraTransaction>(
                        comm_params,
                        &LedgeraCorePublicationTopics::TransactionSubmission
                            .get_publication_topic_str(service.as_ref()),
                        &tins,
                    )
                    .await
                {
                    Err(e) => {
                        return Err(VoterComputationBehaviorError::CouldNotEmitTins(e));
                    }
                    Ok(_) => {
                        log::info!(
                            "As {:?} : submitted 'Tins' transaction for computation instance {:}",
                            LedgeraCoreRoles::VoterComputer,
                            phase1_result
                                .pod
                                .v
                                .function_instance_identifier
                                .to_hexadecimal_string()
                        )
                    }
                }
            }

            // we then wait to receive a delivered "Tins" transaction
            receivers
                .tins_receiver
                .recv()
                .await
                .ok_or(VoterComputationBehaviorError::TinsDeliveryChannelClosed)?
        }
        CollectedAllArguments::ViaReceivingADeliveredTins(tins) => {
            // as we have already received the delivered "Tins", we return it
            tins
        }
    };

    let unknown_arguments_values = {
        // here we have the raw values for all the known arguments
        let mut got_raw_values = HashMap::new();
        // we then get them also for the unknowns
        for (arg_id, arg_ref) in &final_pouav.v.proposed_unknowns_assignment {
            let data_digest = &arg_ref.pos.v.data_digest;
            if let Some(got_data) = cached_retrieved_data.get(data_digest) {
                got_raw_values.insert(*arg_id as usize, got_data.clone());
            } else {
                let got_data = retrieve_data_from_storage(
                    service,
                    comm_api.clone(),
                    comm_params.clone(),
                    &arg_ref.pos,
                )
                .await;
                got_raw_values.insert(*arg_id as usize, got_data.clone());
            }
        }
        Some(got_raw_values)
    };
    let final_tins = LedgeraTransaction::Tins(final_pouav);
    let tins_digest = LedgeraDigest::from_serializable(&final_tins).unwrap();

    Ok(ComputationInstancePhase2Result::new(
        known_arguments_values,
        unknown_arguments_values,
        Some(tins_digest),
    ))
}
