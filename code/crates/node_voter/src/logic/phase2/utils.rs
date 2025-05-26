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
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

use ledgera_comms::{
    comm_api::{LedgeraInternalCommunicationInterface, LedgeraInternalCommunicationParameters},
    comm_session::PubSubNetwork,
};
use ledgera_core_logic::queries::query_data::retrieve_data_from_storage;
use ledgera_core_logic::roles::LedgeraCoreRoles;
use ledgera_pki::manager::{
    KnownParticipantsMap, PublicKeyInfrastructure, SerdeSerializable64BitsSignature,
};
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::app_template::{
    input::LedgeraInputArgument, predicates::LedgeraOperationSingularArgumentPredicate,
};
use ledgera_types::{
    app_template::spec::LedgeraAtomicOperationSpecification,
    requests::rin::LedgeraRequestInputProposal,
};

pub async fn retrieve_known_inputs<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
>(
    comm_api: &Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    comm_params: &Arc<LedgeraInternalCommunicationParameters<PKI>>,
    service: &Arc<LAT>,
    // ***
    arguments: &[LedgeraInputArgument<LAT::Data, LAT::LocalPredicate>],
) -> HashMap<usize, LAT::Data> {
    let retrieved = futures::future::try_join_all(
        arguments
            .iter()
            .enumerate()
            .filter(|(_, arg)| arg.is_concrete())
            .map(|(arg_id, arg)| match arg {
                LedgeraInputArgument::RawValue {
                    is_input_persistent: _,
                    value,
                } => {
                    let value_clone = value.clone();
                    tokio::spawn(async move { (arg_id, value_clone) })
                }
                LedgeraInputArgument::ReferenceToStorage(pos) => {
                    let comm_api_clone = comm_api.clone();
                    let comm_params_clone = comm_params.clone();
                    let service_clone = service.clone();
                    let pos_clone = pos.clone();
                    tokio::spawn(async move {
                        (
                            arg_id,
                            retrieve_data_from_storage(
                                &service_clone,
                                comm_api_clone,
                                comm_params_clone,
                                &pos_clone,
                            )
                            .await,
                        )
                    })
                }
                LedgeraInputArgument::Unknown(_) => {
                    unreachable!()
                }
            }),
    )
    .await
    .unwrap();
    retrieved
        .into_iter() /* .map(|(i,v)| (i,Rc::new(v)))*/
        .collect()
}

pub fn is_rin_unknown_input_proposal_initially_valid<PKI: PublicKeyInfrastructure>(
    operation_unknown_arguments_indices: &BTreeSet<u32>,
    rin: &LedgeraRequestInputProposal,
    known_participants: &KnownParticipantsMap<PKI::VerifyingKey>,
    threshold: u32,
) -> bool {
    // Rin not valid if the argument it provides is not for any indices
    if rin.argument_indices.is_empty() {
        log::info!(
            "As {:?} : invalid Rin : it is not proposed at any argument index",
            LedgeraCoreRoles::VoterComputer
        );
        return false;
    }
    // Rin not valid if the argument it provides is not for indices that are unknowns of the computation spec
    if !rin
        .argument_indices
        .iter()
        .all(|idx| operation_unknown_arguments_indices.contains(idx))
    {
        log::info!(
            "As {:?} : invalid Rin : it is not proposed at an argument index that corresponds to an unknown",
            LedgeraCoreRoles::VoterComputer
        );
        return false;
    }
    // Def. 11: lps must be a valid LP_S (f+1 signatures). Checking here avoids pointless
    // Q_sto round-trips to all storers for Rins whose LP_S is forged or under-signed.
    if let Err(e) = rin
        .input_data
        .verify_proof_of_shipment_to_storage::<PKI>(known_participants, threshold)
    {
        log::info!(
            "As {:?} : invalid Rin : LP_S quorum is not valid : {:?}",
            LedgeraCoreRoles::VoterComputer,
            e
        );
        return false;
    }
    true
}

pub fn is_rin_locally_valid<LAT: LedgeraApplicationTemplate>(
    function_instance_identifier: &SerdeSerializable64BitsSignature,
    comp_spec: &LedgeraAtomicOperationSpecification<LAT>,
    rin: &LedgeraRequestInputProposal,
    rin_referred_data: &LAT::Data,
) -> bool {
    for idx in &rin.argument_indices {
        let pred: &LAT::LocalPredicate = comp_spec
            .arguments
            .get(*idx as usize)
            .unwrap()
            .get_predicate()
            .unwrap();
        match pred.is_valid_for(rin_referred_data, function_instance_identifier) {
            Ok(is_valid) => {
                if !is_valid {
                    return false;
                }
            }
            Err(_) => {
                return false;
            }
        }
    }
    true
}
