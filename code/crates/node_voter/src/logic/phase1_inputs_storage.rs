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

use std::sync::Arc;

use ledgera_comms::{
    comm_api::{LedgeraInternalCommunicationInterface, LedgeraInternalCommunicationParameters},
    comm_session::PubSubNetwork,
};
use ledgera_pki::{manager::PublicKeyInfrastructure, message::SignatureEntry};
use ledgera_types::{
    app_template::{input::LedgeraInputArgument, template::LedgeraApplicationTemplate},
    digest::LedgeraDigest,
    requests::rsto::LedgeraServerSideStorageRequest,
    votes::vsto::{LedgeraVoteStored, PersistentDataKind},
};

use crate::{
    logic::{
        outputs::ComputationInstancePhase1Result, storage_logic::emit_and_collect_storage_quorums,
    },
    management::error::VoterComputationBehaviorError,
};

pub async fn phase1_inputs_storage_logic<PKI, Sess, LAT>(
    comm_api: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    comm_params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
    service: Arc<LAT>,
    phase1_result: Arc<ComputationInstancePhase1Result<LAT>>,
    mut inputs_vstored_receiver: tokio::sync::mpsc::Receiver<(SignatureEntry, LedgeraVoteStored)>,
) -> Result<(), VoterComputationBehaviorError<Sess, LAT>>
where
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
{
    let mut rsto_requests = vec![];
    for (arg_id, arg) in phase1_result.op_spec.arguments.iter().enumerate() {
        if let LedgeraInputArgument::RawValue {
            is_input_persistent,
            value: data_value,
        } = arg
        {
            if *is_input_persistent {
                let data_digest = LedgeraDigest::from_serializable(data_value).unwrap();
                let data_kind = PersistentDataKind::Input(arg_id as u32);
                rsto_requests.push((
                    LedgeraServerSideStorageRequest::new(
                        data_value.clone(),
                        data_kind.clone(),
                        phase1_result.pod.clone(),
                        None,
                    ),
                    LedgeraVoteStored::new(
                        phase1_result.pod.v.function_instance_identifier.clone(),
                        data_digest,
                        data_kind,
                    ),
                ));
            }
        }
    }
    emit_and_collect_storage_quorums(
        &comm_api,
        &comm_params,
        &service,
        rsto_requests,
        &mut inputs_vstored_receiver,
    )
    .await
}
