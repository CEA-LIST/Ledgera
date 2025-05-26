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
    app_template::template::LedgeraApplicationTemplate,
    requests::rsto::LedgeraServerSideStorageRequest,
    votes::vout::LedgeraFunctionInstanceOutputKind,
    votes::vsto::{LedgeraVoteStored, PersistentDataKind},
};

use crate::{
    logic::{
        outputs::{ComputationInstancePhase1Result, ComputationInstancePhase3Result},
        storage_logic::emit_and_collect_storage_quorums,
    },
    management::error::VoterComputationBehaviorError,
};

pub async fn phase4_output_storage_logic<PKI, Sess, LAT>(
    comm_api: &Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    comm_params: &Arc<LedgeraInternalCommunicationParameters<PKI>>,
    service: &Arc<LAT>,
    phase1_result: &ComputationInstancePhase1Result<LAT>,
    phase3_result_opt: Option<ComputationInstancePhase3Result<LAT::Data>>,
    mut output_vstored_receiver: tokio::sync::mpsc::Receiver<(SignatureEntry, LedgeraVoteStored)>,
) -> Result<(), VoterComputationBehaviorError<Sess, LAT>>
where
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
{
    let mut rsto_requests = vec![];
    if let Some(phase3_result) = phase3_result_opt {
        match &phase3_result.poi.v.result_kind {
            LedgeraFunctionInstanceOutputKind::TaggedInputs => {
                // no output to store
            }
            LedgeraFunctionInstanceOutputKind::ComputedOutput {
                is_output_persistent,
                output_digest,
            } => {
                if *is_output_persistent {
                    let data_value = phase3_result.result_value.unwrap();
                    rsto_requests.push((
                        LedgeraServerSideStorageRequest::new(
                            data_value,
                            PersistentDataKind::Output,
                            phase1_result.pod.clone(),
                            Some(phase3_result.poi.clone()),
                        ),
                        LedgeraVoteStored::new(
                            phase1_result.pod.v.function_instance_identifier.clone(),
                            output_digest.clone(),
                            PersistentDataKind::Output,
                        ),
                    ));
                }
            }
        }
    }
    emit_and_collect_storage_quorums(
        comm_api,
        comm_params,
        service,
        rsto_requests,
        &mut output_vstored_receiver,
    )
    .await
}
