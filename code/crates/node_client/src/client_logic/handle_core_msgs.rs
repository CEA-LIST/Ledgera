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

use ledgera_comms::comm_api::LedgeraInternalCommunicationParameters;
use ledgera_core_logic::roles::LedgeraCoreRoles;
use ledgera_pki::{manager::PublicKeyInfrastructure, message::SignatureEntry};
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::messages::{
    deliver::LedgeraTransactionDeliveryNotification, nres::LedgeraComputationResultNotification,
};
use ledgera_types::requests::rfun::LedgeraRequestFunctionInstanceProposal;
use ledgera_types::traits::LedgeraQuorumContainingMessage;
use ledgera_types::transactions::LedgeraTransaction;

use crate::comms::feedback_from_core_client::{
    ValidatedComputationInstance, ValidatedCoreFeedbackMessage,
};

pub async fn client_handling_of_core_messages<
    PKI: PublicKeyInfrastructure,
    LAT: LedgeraApplicationTemplate,
>(
    comm_params_ref: Arc<LedgeraInternalCommunicationParameters<PKI>>,
    mut rfun_receiver: tokio::sync::mpsc::Receiver<(
        LedgeraRequestFunctionInstanceProposal<LAT>,
        SignatureEntry,
    )>,
    mut delivered_txs_receiver: tokio::sync::mpsc::Receiver<(
        LedgeraTransactionDeliveryNotification,
        SignatureEntry,
    )>,
    mut client_notifications_receiver: tokio::sync::mpsc::Receiver<(
        LedgeraComputationResultNotification<LAT::Data>,
        SignatureEntry,
    )>,
    validated_core_msgs_sender: tokio::sync::mpsc::Sender<ValidatedCoreFeedbackMessage<LAT>>,
) {
    // KNOWN LIMITATION: both maps grow without bound if matching counterpart messages never arrive.
    //
    // pending_rfuns holds validated Rfun messages whose corresponding delivered Tfun has not yet
    // been seen. pending_tfuns holds delivered Tfun messages whose corresponding Rfun has not yet
    // been seen. Under normal operation the maps are small (bounded by in-flight computation
    // instances). Under adversarial conditions a Byzantine client can flood the Rfun topic with
    // valid messages (an Rfun only requires the client's own signature, not a multi-party quorum),
    // causing pending_rfuns to grow without bound and eventually OOM the process.
    //
    // A naive capacity cap does not work: dropping an Rfun entry causes its eventual Tfun to
    // strand in pending_tfuns permanently (and vice-versa), so capping one map independently
    // makes the other worse.
    //
    // The correct fix is to filter the Rfun subscription at the source so that this client only
    // receives Rfun messages for computation instances it submitted itself. That would eliminate
    // the Byzantine flooding vector entirely, since a client already knows its own computation IDs
    // from the SerdeSerializable64BitsSignature returned by CoreClientRuntime::compute_function().
    // This change belongs in client_behavior.rs at the subscribe_to_topic_and_deserialize_as call
    // for LedgeraCorePublicationTopics::Rfun, not here.
    let mut pending_rfuns = HashMap::new();
    let mut pending_tfuns = HashMap::new();
    loop {
        tokio::select! {
            Some((rfun,rfun_signature)) = rfun_receiver.recv() => {
                match rfun
                    .verify_vote_quorums::<PKI>(
                        &comm_params_ref.known_participants,
                        comm_params_ref.byzantine_threshold,
                    ) {
                    Ok(_) => {
                        log::info!(
                            "As {:?} : acknowledging Rfun",
                            LedgeraCoreRoles::Client
                        );
                        match pending_tfuns.remove(&rfun_signature.serializable_signature) {
                            None => {
                                // See KNOWN LIMITATION above: pending_rfuns is unbounded.
                                pending_rfuns.insert(rfun_signature.serializable_signature,rfun);
                            },
                            Some(tfun) => {
                                let validated = ValidatedCoreFeedbackMessage::ValidatedComputationInstance(
                                    ValidatedComputationInstance::new(
                                        rfun_signature.serializable_signature,
                                        rfun,
                                        tfun
                                    )
                                );
                                if validated_core_msgs_sender.send(validated).await.is_err() {
                                    log::warn!("As {:?} : application receiver dropped, stopping client message handler", LedgeraCoreRoles::Client);
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "As {:?} : received a Rfun with incorrect quorum(s) inside : {:?}",
                            LedgeraCoreRoles::Client,
                            e
                        );
                    }
                }
            },
            Some((dlvrd_tx,_)) = delivered_txs_receiver.recv() => {
                match dlvrd_tx.transaction
                    .verify_vote_quorums::<PKI>(
                        &comm_params_ref.known_participants,
                        comm_params_ref.byzantine_threshold,
                    ) {
                    Ok(_) => {
                        log::info!(
                            "As {:?} : acknowledging delivered {:} transaction",
                            LedgeraCoreRoles::Client,
                            dlvrd_tx.transaction.get_transaction_kind()
                        );
                        match dlvrd_tx.transaction {
                            LedgeraTransaction::Tsto(anchor_proof_of_storage) => {
                                let validated = ValidatedCoreFeedbackMessage::DeliveredTsto(
                                    anchor_proof_of_storage
                                );
                                if validated_core_msgs_sender.send(validated).await.is_err() {
                                    log::warn!("As {:?} : application receiver dropped, stopping client message handler", LedgeraCoreRoles::Client);
                                    return;
                                }
                            },
                            LedgeraTransaction::Tfun(anchor_computation_instance_declaration) => {
                                match pending_rfuns.remove(&anchor_computation_instance_declaration.v.function_instance_identifier) {
                                    None => {
                                        // See KNOWN LIMITATION above: pending_tfuns is unbounded.
                                        pending_tfuns.insert(
                                            anchor_computation_instance_declaration.v.function_instance_identifier.clone(),
                                            anchor_computation_instance_declaration
                                        );
                                    },
                                    Some(rfun) => {
                                        let validated = ValidatedCoreFeedbackMessage::ValidatedComputationInstance(
                                            ValidatedComputationInstance::new(
                                                anchor_computation_instance_declaration.v.function_instance_identifier.clone(),
                                                rfun,
                                                anchor_computation_instance_declaration
                                            )
                                        );
                                        if validated_core_msgs_sender.send(validated).await.is_err() {
                                            log::warn!("As {:?} : application receiver dropped, stopping client message handler", LedgeraCoreRoles::Client);
                                            return;
                                        }
                                    }
                                }
                            },
                            LedgeraTransaction::Tins(anchor_agreement_on_unknown_input_arguments) => {
                                let validated = ValidatedCoreFeedbackMessage::DeliveredTins(
                                    anchor_agreement_on_unknown_input_arguments
                                );
                                if validated_core_msgs_sender.send(validated).await.is_err() {
                                    log::warn!("As {:?} : application receiver dropped, stopping client message handler", LedgeraCoreRoles::Client);
                                    return;
                                }
                            },
                            LedgeraTransaction::Tout(anchor_proof_of_integrity) => {
                                let validated = ValidatedCoreFeedbackMessage::DeliveredTout(
                                    anchor_proof_of_integrity
                                );
                                if validated_core_msgs_sender.send(validated).await.is_err() {
                                    log::warn!("As {:?} : application receiver dropped, stopping client message handler", LedgeraCoreRoles::Client);
                                    return;
                                }
                            },
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "As {:?} : received a delivered transaction with incorrect quorum(s) inside : {:?}",
                            LedgeraCoreRoles::Client,
                            e
                        );
                    }
                }
            },
            Some((client_notif,_)) = client_notifications_receiver.recv() => {
                match client_notif
                    .verify_vote_quorums::<PKI>(
                        &comm_params_ref.known_participants,
                        comm_params_ref.byzantine_threshold,
                    ) {
                    Ok(_) => {
                        log::info!(
                            "As {:?} : acknowledging computation result notification",
                            LedgeraCoreRoles::Client
                        );
                        let validated = ValidatedCoreFeedbackMessage::Nout(
                            client_notif
                        );
                        if validated_core_msgs_sender.send(validated).await.is_err() {
                            log::warn!("As {:?} : application receiver dropped, stopping client message handler", LedgeraCoreRoles::Client);
                            return;
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "As {:?} : received a client notification with incorrect quorum(s) inside : {:?}",
                            LedgeraCoreRoles::Client,
                            e
                        );
                    }
                }
            }
        }
    }
}
