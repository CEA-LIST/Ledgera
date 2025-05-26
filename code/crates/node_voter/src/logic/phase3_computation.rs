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

use crate::logic::outputs::ComputationInstancePhase1Result;
use crate::management::channels::PerInstanceVoterBehaviorPhase3Receivers;
use crate::{
    logic::outputs::{ComputationInstancePhase2Result, ComputationInstancePhase3Result},
    management::error::VoterComputationBehaviorError,
};
use ledgera_comms::{
    comm_api::{LedgeraInternalCommunicationInterface, LedgeraInternalCommunicationParameters},
    comm_session::PubSubNetwork,
};
use ledgera_core_logic::{
    quorum::collect_quorum, roles::LedgeraCoreRoles, topics::LedgeraCorePublicationTopics,
};
use ledgera_pki::{manager::PublicKeyInfrastructure, quorum::QuorumOfSignatures};
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::digest::LedgeraDigest;
use ledgera_types::messages::nres::LedgeraComputationResultNotification;
use ledgera_types::transactions::LedgeraTransaction;
use ledgera_types::votes::vout::LedgeraVoteFunctionOutput;
use ledgera_types::{
    app_template::operation::{LedgeraAtomicComputation, LedgeraAtomicOperation},
    proofs::proof_of_integrity::ProofOfOperationIntegrity,
    votes::vout::LedgeraFunctionInstanceOutputKind,
};

pub async fn phase3_computation_logic<
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
    phase2_result: ComputationInstancePhase2Result<LAT>,
    // ***
    // asynch receivers
    mut receivers: PerInstanceVoterBehaviorPhase3Receivers,
) -> Result<
    Option<ComputationInstancePhase3Result<LAT::Data>>,
    VoterComputationBehaviorError<Sess, LAT>,
> {
    // there is no computation done/Vout emitted/Tout produced iff:
    // - the operation is a tag_inputs and there is no unknown input
    if phase1_result.op_spec.operation.is_tag_inputs()
        && phase1_result.pod.v.unknown_arguments_indices.is_empty()
    {
        return Ok(None);
    }

    // if there are some unknowns, we need to collect their digests to emit the Vout
    let unknow_arguments_values;
    let unknowns_agreement_ref: Option<LedgeraDigest>;
    if phase1_result.pod.v.unknown_arguments_indices.is_empty() {
        unknowns_agreement_ref = None;
        unknow_arguments_values = phase2_result.unknow_arguments_values;
    } else {
        let unknow_arguments_values_unwrapped = phase2_result.unknow_arguments_values.unwrap();
        unknowns_agreement_ref = Some(phase2_result.tins_digest.unwrap().clone());
        unknow_arguments_values = Some(unknow_arguments_values_unwrapped);
    }

    // ========================================================================================
    // ==== Phase 3 / Part 1 :
    // ====  - compute the result locally
    // ====  - and then emit a Vout
    // ========================================================================================
    let mut result_raw;
    let result_kind;
    if phase1_result.op_spec.operation.is_tag_inputs() {
        result_raw = None;
        result_kind = LedgeraFunctionInstanceOutputKind::TaggedInputs
    } else {
        let unknow_arguments_values_unwrapped = unknow_arguments_values.unwrap_or_default();
        let mut final_array = Vec::new();
        for arg_idx in 0..phase1_result.op_spec.arguments.len() {
            if let Some(v) = phase2_result.know_arguments_values.get(&arg_idx) {
                final_array.push(v.clone());
            } else {
                final_array.push(
                    unknow_arguments_values_unwrapped
                        .get(&arg_idx)
                        .unwrap()
                        .clone(),
                );
            }
        }
        log::info!(
            "As {:?} : performing computation {}\nlocally with operator {:?}\n on data {:?}",
            LedgeraCoreRoles::VoterComputer,
            phase1_result
                .pod
                .v
                .function_instance_identifier
                .to_hexadecimal_string(),
            phase1_result.op_spec.operation,
            final_array
        );
        let cmp_res = match &phase1_result.op_spec.operation {
            LedgeraAtomicOperation::ComputeOutput {
                is_output_persistent: _,
                comp: computation,
            } => match computation.compute(final_array).await {
                Ok(val) => Ok(val),
                Err(e) => Err(VoterComputationBehaviorError::ErrorWhenComputingLocalResult(e)),
            },
            _ => Err(VoterComputationBehaviorError::TryingComputeOnATagOperation),
        }?;
        log::info!(
            "As {:?} : performing {} computation\ngot local result {:?}",
            LedgeraCoreRoles::VoterComputer,
            phase1_result
                .pod
                .v
                .function_instance_identifier
                .to_hexadecimal_string(),
            cmp_res
        );
        let result_digest = LedgeraDigest::from_serializable(&cmp_res).unwrap();
        result_raw = Some(cmp_res);
        result_kind = LedgeraFunctionInstanceOutputKind::ComputedOutput {
            is_output_persistent: phase1_result.op_spec.operation.is_output_persistent(),
            output_digest: result_digest,
        };
    };

    let vout_vote = LedgeraVoteFunctionOutput::new(
        phase1_result.pod.v.function_instance_identifier.clone(),
        unknowns_agreement_ref,
        result_kind,
    );
    {
        let mut comm_sess = comm_api.lock().await;
        match comm_sess
            .serialize_and_publish_on_topic::<LedgeraVoteFunctionOutput>(
                comm_params,
                &LedgeraCorePublicationTopics::Vout.get_publication_topic_str(service.as_ref()),
                &vout_vote,
            )
            .await
        {
            Err(e) => {
                return Err(VoterComputationBehaviorError::CouldNotEmitVout(e));
            }
            Ok(_) => {
                log::info!(
                    "As {:?} : emitted Vout vote for computation instance {:?}",
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

    // ========================================================================================
    // ==== Phase 3 / Part 2 :
    // ====  - collect f+1 Vout on the same digest as the local result digest
    // ========================================================================================

    let (filtered_vout_sender, filtered_vout_receiver) = tokio::sync::mpsc::channel(128);
    let mut wait_vout_quorum: Pin<Box<dyn Future<Output = Option<QuorumOfSignatures>> + Send>> =
        Box::pin(collect_quorum::<PKI>(
            bincode::serialize(&vout_vote).unwrap(),
            filtered_vout_receiver,
            comm_params.byzantine_threshold as usize,
        ));
    let got_vout_quorum: QuorumOfSignatures;
    'wait_for_vout_votes: loop {
        tokio::select! {
            Some((sig,other_vout)) = receivers.vout_receiver.recv() => {
                if other_vout == vout_vote {
                    let _ = filtered_vout_sender.send(sig).await;
                }
            }
            Some(q) = wait_vout_quorum.as_mut() => {
                got_vout_quorum = q;
                break 'wait_for_vout_votes;
            }
        }
    }

    // we produce the proof of integrity
    let mut poi = ProofOfOperationIntegrity::new(vout_vote, got_vout_quorum);

    // ========================================================================================
    // ==== Phase 3 / Part 3 :
    // ====  - send a Nout and a Tout
    // ========================================================================================
    // if there is a result, send it as a 'Nout'
    if let Some(cmp_res) = result_raw {
        let nres = LedgeraComputationResultNotification::new(cmp_res, poi);
        {
            let mut comm_sess = comm_api.lock().await;
            match comm_sess
                .serialize_and_publish_on_topic::<LedgeraComputationResultNotification<LAT::Data>>(
                    comm_params,
                    &LedgeraCorePublicationTopics::Nout(hex::encode(phase1_result.sender))
                        .get_publication_topic_str(service.as_ref()),
                    &nres,
                )
                .await
            {
                Err(e) => {
                    return Err(VoterComputationBehaviorError::CouldNotEmitNout(e));
                }
                Ok(_) => {
                    log::info!(
                        "As {:?} : emitted Nout vote for computation instance {:?} to client {:?}",
                        LedgeraCoreRoles::VoterComputer,
                        phase1_result
                            .pod
                            .v
                            .function_instance_identifier
                            .to_hexadecimal_string(),
                        hex::encode(phase1_result.sender)
                    )
                }
            }
        }
        result_raw = Some(nres.result_value);
        poi = nres.poi;
    }

    let tout = LedgeraTransaction::Tout(poi);
    {
        let mut comm_sess = comm_api.lock().await;
        match comm_sess
            .serialize_and_publish_on_topic::<LedgeraTransaction>(
                comm_params,
                &LedgeraCorePublicationTopics::TransactionSubmission
                    .get_publication_topic_str(service.as_ref()),
                &tout,
            )
            .await
        {
            Err(e) => {
                return Err(VoterComputationBehaviorError::CouldNotEmitTout(e));
            }
            Ok(_) => {
                log::info!(
                    "As {:?} : submitted Tout vote for computation instance {:?}",
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

    if let LedgeraTransaction::Tout(poi) = tout {
        Ok(Some(ComputationInstancePhase3Result::new(poi, result_raw)))
    } else {
        unreachable!();
    }
}
