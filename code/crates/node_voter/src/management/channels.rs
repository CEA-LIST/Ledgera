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

use ledgera_pki::message::SignatureEntry;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::proofs::proof_of_declaration::ProofOfFunctionDeclaration;
use ledgera_types::proofs::proof_of_integrity::ProofOfOperationIntegrity;
use ledgera_types::proofs::proof_of_unknown_arguments_assignment_verification::ProofOfUnknownArgumentsAssignmentVerification;
use ledgera_types::requests::rfun::LedgeraRequestFunctionInstanceProposal;
use ledgera_types::requests::rin::LedgeraRequestInputProposal;
use ledgera_types::votes::vins::LedgeraVoteIns;
use ledgera_types::votes::vout::LedgeraVoteFunctionOutput;
use ledgera_types::votes::vsto::LedgeraVoteStored;

pub struct PerInstanceVoterBehaviorSenders<LAT: LedgeraApplicationTemplate> {
    pub rfun_sender:
        tokio::sync::mpsc::Sender<(SignatureEntry, LedgeraRequestFunctionInstanceProposal<LAT>)>,
    pub vfun_sender: tokio::sync::mpsc::Sender<SignatureEntry>,
    pub tcomp_sender: tokio::sync::mpsc::Sender<ProofOfFunctionDeclaration>,
    // ***
    pub rin_sender: tokio::sync::mpsc::Sender<(SignatureEntry, LedgeraRequestInputProposal)>,
    pub vins_sender: tokio::sync::mpsc::Sender<(SignatureEntry, LedgeraVoteIns)>,
    pub tins_sender: tokio::sync::mpsc::Sender<ProofOfUnknownArgumentsAssignmentVerification>,
    // ***
    pub vout_sender: tokio::sync::mpsc::Sender<(SignatureEntry, LedgeraVoteFunctionOutput)>,
    pub tout_sender: tokio::sync::mpsc::Sender<ProofOfOperationIntegrity>,
    // ***
    pub inputs_vstored_sender: tokio::sync::mpsc::Sender<(SignatureEntry, LedgeraVoteStored)>,
    pub output_vstored_sender: tokio::sync::mpsc::Sender<(SignatureEntry, LedgeraVoteStored)>,
}

pub struct PerInstanceVoterBehaviorReceivers<LAT: LedgeraApplicationTemplate> {
    pub phase1: PerInstanceVoterBehaviorPhase1Receivers<LAT>,
    pub phase2: PerInstanceVoterBehaviorPhase2Receivers,
    pub phase3: PerInstanceVoterBehaviorPhase3Receivers,
    pub inputs_vstored_receiver: tokio::sync::mpsc::Receiver<(SignatureEntry, LedgeraVoteStored)>,
    pub output_vstored_receiver: tokio::sync::mpsc::Receiver<(SignatureEntry, LedgeraVoteStored)>,
}

pub struct PerInstanceVoterBehaviorPhase1Receivers<LAT: LedgeraApplicationTemplate> {
    pub rfun_receiver:
        tokio::sync::mpsc::Receiver<(SignatureEntry, LedgeraRequestFunctionInstanceProposal<LAT>)>,
    pub vfun_receiver: tokio::sync::mpsc::Receiver<SignatureEntry>,
    pub _tcomp_receiver: tokio::sync::mpsc::Receiver<ProofOfFunctionDeclaration>,
}

pub struct PerInstanceVoterBehaviorPhase2Receivers {
    pub rin_sender_clone: tokio::sync::mpsc::Sender<(SignatureEntry, LedgeraRequestInputProposal)>,
    pub rin_receiver: tokio::sync::mpsc::Receiver<(SignatureEntry, LedgeraRequestInputProposal)>,
    pub vins_receiver: tokio::sync::mpsc::Receiver<(SignatureEntry, LedgeraVoteIns)>,
    pub tins_receiver: tokio::sync::mpsc::Receiver<ProofOfUnknownArgumentsAssignmentVerification>,
}

pub struct PerInstanceVoterBehaviorPhase3Receivers {
    pub vout_receiver: tokio::sync::mpsc::Receiver<(SignatureEntry, LedgeraVoteFunctionOutput)>,
    pub _tout_receiver: tokio::sync::mpsc::Receiver<ProofOfOperationIntegrity>,
}

pub enum InitialMessageCreatingInstanceState<LAT: LedgeraApplicationTemplate> {
    Rfun(SignatureEntry, LedgeraRequestFunctionInstanceProposal<LAT>),
    Vfun(SignatureEntry),
    Tfun(ProofOfFunctionDeclaration),
    // ***
    Rin(SignatureEntry, LedgeraRequestInputProposal),
    Vins(SignatureEntry, LedgeraVoteIns),
    Tins(ProofOfUnknownArgumentsAssignmentVerification),
    // ***
    Vout(SignatureEntry, LedgeraVoteFunctionOutput),
    Tout(ProofOfOperationIntegrity),
    // ***
    VstoInput(SignatureEntry, LedgeraVoteStored),
    VstoOutput(SignatureEntry, LedgeraVoteStored),
}

pub async fn initiate_computation_instance_state_on_voter<LAT: LedgeraApplicationTemplate>(
    initial_message: InitialMessageCreatingInstanceState<LAT>,
) -> (
    PerInstanceVoterBehaviorSenders<LAT>,
    PerInstanceVoterBehaviorReceivers<LAT>,
) {
    // ***
    let (rfun_sender, rfun_receiver) = tokio::sync::mpsc::channel(128);
    let (vfun_sender, vfun_receiver) = tokio::sync::mpsc::channel(128);
    let (tcomp_sender, tcomp_receiver) = tokio::sync::mpsc::channel(128);
    // ***
    let (rin_sender, rin_receiver) = tokio::sync::mpsc::channel(128);
    let (vins_sender, vins_receiver) = tokio::sync::mpsc::channel(128);
    let (tins_sender, tins_receiver) = tokio::sync::mpsc::channel(128);
    // ***
    let (vout_sender, vout_receiver) = tokio::sync::mpsc::channel(128);
    let (tout_sender, tout_receiver) = tokio::sync::mpsc::channel(128);
    // ***
    let (inputs_vstored_sender, inputs_vstored_receiver) = tokio::sync::mpsc::channel(128);
    let (output_vstored_sender, output_vstored_receiver) = tokio::sync::mpsc::channel(128);
    match initial_message {
        // ***
        InitialMessageCreatingInstanceState::Rfun(x, y) => {
            let _ = rfun_sender.send((x, y)).await;
        }
        InitialMessageCreatingInstanceState::Vfun(x) => {
            let _ = vfun_sender.send(x).await;
        }
        InitialMessageCreatingInstanceState::Tfun(x) => {
            let _ = tcomp_sender.send(x).await;
        }
        // ***
        InitialMessageCreatingInstanceState::Rin(x, y) => {
            let _ = rin_sender.send((x, y)).await;
        }
        InitialMessageCreatingInstanceState::Vins(x, y) => {
            let _ = vins_sender.send((x, y)).await;
        }
        InitialMessageCreatingInstanceState::Tins(x) => {
            let _ = tins_sender.send(x).await;
        }
        // ***
        InitialMessageCreatingInstanceState::Vout(x, y) => {
            let _ = vout_sender.send((x, y)).await;
        }
        InitialMessageCreatingInstanceState::Tout(x) => {
            let _ = tout_sender.send(x).await;
        }
        // ***
        InitialMessageCreatingInstanceState::VstoInput(x, y) => {
            let _ = inputs_vstored_sender.send((x, y)).await;
        }
        InitialMessageCreatingInstanceState::VstoOutput(x, y) => {
            let _ = output_vstored_sender.send((x, y)).await;
        }
    }
    let per_instance_senders = PerInstanceVoterBehaviorSenders::<LAT> {
        rfun_sender,
        vfun_sender,
        tcomp_sender,
        rin_sender: rin_sender.clone(),
        vins_sender,
        tins_sender,
        vout_sender,
        tout_sender,
        inputs_vstored_sender,
        output_vstored_sender,
    };

    let per_instance_receivers = PerInstanceVoterBehaviorReceivers {
        phase1: PerInstanceVoterBehaviorPhase1Receivers {
            rfun_receiver,
            vfun_receiver,
            _tcomp_receiver: tcomp_receiver,
        },
        phase2: PerInstanceVoterBehaviorPhase2Receivers {
            rin_sender_clone: rin_sender,
            rin_receiver,
            vins_receiver,
            tins_receiver,
        },
        phase3: PerInstanceVoterBehaviorPhase3Receivers {
            vout_receiver,
            _tout_receiver: tout_receiver,
        },
        inputs_vstored_receiver,
        output_vstored_receiver,
    };
    (per_instance_senders, per_instance_receivers)
}
