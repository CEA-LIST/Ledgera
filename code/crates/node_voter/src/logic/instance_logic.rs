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
use ledgera_core_logic::roles::LedgeraCoreRoles;
use ledgera_pki::manager::PublicKeyInfrastructure;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;

use crate::logic::phase1_inputs_storage::phase1_inputs_storage_logic;
use crate::logic::phase2_unknowns_collection::phase2_unknowns_collection_logic;
use crate::logic::phase3_computation::phase3_computation_logic;
use crate::logic::phase4_output_storage::phase4_output_storage_logic;
use crate::{
    logic::phase1_access_control::phase1_access_control_logic,
    management::{
        channels::PerInstanceVoterBehaviorReceivers, error::VoterComputationBehaviorError,
    },
};

pub async fn run_computation_instance_logic<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
>(
    comp_instance_id_str: String,
    comm_api: Arc<tokio::sync::Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
    comm_params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
    service: Arc<LAT>,
    receivers: PerInstanceVoterBehaviorReceivers<LAT>,
) -> Result<(), VoterComputationBehaviorError<Sess, LAT>> {
    log::info!(
        "As {:?} : starting phase 1 for computation instance {}",
        LedgeraCoreRoles::VoterComputer,
        comp_instance_id_str
    );
    let phase1_result = Arc::new(
        phase1_access_control_logic(&comm_api, &comm_params, &service, receivers.phase1).await?,
    );

    log::info!(
        "As {:?} : starting phase 1 inputs storage in background for computation instance {}",
        LedgeraCoreRoles::VoterComputer,
        comp_instance_id_str
    );
    let inputs_storage_handle = {
        let comm_api = comm_api.clone();
        let comm_params = comm_params.clone();
        let service = service.clone();
        let phase1_result = phase1_result.clone();
        tokio::spawn(async move {
            phase1_inputs_storage_logic(
                comm_api,
                comm_params,
                service,
                phase1_result,
                receivers.inputs_vstored_receiver,
            )
            .await
        })
    };

    log::info!(
        "As {:?} : starting phase 2 for computation instance {}",
        LedgeraCoreRoles::VoterComputer,
        comp_instance_id_str
    );
    let phase2_result = phase2_unknowns_collection_logic(
        &comm_api,
        &comm_params,
        &service,
        &phase1_result,
        receivers.phase2,
    )
    .await?;

    log::info!(
        "As {:?} : starting phase 3 for computation instance {}",
        LedgeraCoreRoles::VoterComputer,
        comp_instance_id_str
    );
    let phase3_result = phase3_computation_logic(
        &comm_api,
        &comm_params,
        &service,
        &phase1_result,
        phase2_result,
        receivers.phase3,
    )
    .await?;

    log::info!(
        "As {:?} : starting phase 4 output storage for computation instance {}",
        LedgeraCoreRoles::VoterComputer,
        comp_instance_id_str
    );
    phase4_output_storage_logic(
        &comm_api,
        &comm_params,
        &service,
        &phase1_result,
        phase3_result,
        receivers.output_vstored_receiver,
    )
    .await?;

    match inputs_storage_handle.await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            log::warn!(
                "As {:?} : input storage task failed for computation instance {} : {:?}",
                LedgeraCoreRoles::VoterComputer,
                comp_instance_id_str,
                e
            );
        }
        Err(e) => {
            log::warn!(
                "As {:?} : input storage task panicked for computation instance {} : {:?}",
                LedgeraCoreRoles::VoterComputer,
                comp_instance_id_str,
                e
            );
        }
    }

    Ok(())
}
