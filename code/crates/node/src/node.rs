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

use ledgera_comms::comm_api::{
    LedgeraInternalCommunicationInterface, LedgeraInternalCommunicationParameters,
};
use ledgera_comms::comm_session::PubSubNetwork;
use ledgera_comms::error::LedgeraCommunicationError;
use ledgera_core_logic::roles::LedgeraCoreRoles;
use ledgera_node_client::client_logic::client_behavior::LedgeraClientRunOutput;
use ledgera_node_client::client_logic::client_state::LedgeraClientNodeState;
use ledgera_node_logger::orderer_node_state::LedgeraOrdererNodeState;
use ledgera_node_store::storage_node_state::LedgeraStorageNodeState;
use ledgera_node_voter::behavior::LedgeraVoterBehavior;
use ledgera_pki::manager::PublicKeyInfrastructure;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct LedgerNodeState<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate,
> {
    as_storage_node: Option<LedgeraStorageNodeState<PKI, Sess, LAT>>,
    as_client_node: Option<LedgeraClientNodeState<PKI, Sess, LAT>>,
    as_voter_node: Option<LedgeraVoterBehavior<PKI, Sess, LAT>>,
    as_orderer_node: Option<LedgeraOrdererNodeState<PKI, Sess, LAT>>,
}

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork, LAT: LedgeraApplicationTemplate>
    LedgerNodeState<PKI, Sess, LAT>
{
    pub async fn new(
        comm_params: Arc<LedgeraInternalCommunicationParameters<PKI>>,
        service: Arc<LAT>,
        comm_api_ref: Arc<Mutex<LedgeraInternalCommunicationInterface<PKI, Sess>>>,
        roles: HashSet<LedgeraCoreRoles>,
    ) -> Result<Self, LedgeraCommunicationError<Sess::CommRuntimeError>> {
        let as_storage_node = if roles.contains(&LedgeraCoreRoles::PersistentStorage) {
            Some(LedgeraStorageNodeState::<PKI, Sess, LAT>::new(
                comm_api_ref.clone(),
                comm_params.clone(),
                service.clone(),
            ))
        } else {
            None
        };
        let as_client_node = if roles.contains(&LedgeraCoreRoles::Client) {
            Some(LedgeraClientNodeState::<PKI, Sess, LAT>::new(
                comm_api_ref.clone(),
                comm_params.clone(),
                service.clone(),
            ))
        } else {
            None
        };
        let as_voter_node = if roles.contains(&LedgeraCoreRoles::VoterComputer) {
            Some(LedgeraVoterBehavior::<PKI, Sess, LAT>::new(
                comm_api_ref.clone(),
                comm_params.clone(),
                service.clone(),
            ))
        } else {
            None
        };
        let as_orderer_node = if roles.contains(&LedgeraCoreRoles::SecureLogger) {
            Some(LedgeraOrdererNodeState::<PKI, Sess, LAT>::new(
                comm_api_ref.clone(),
                comm_params.clone(),
                service.clone(),
            ))
        } else {
            None
        };
        Ok(Self {
            as_storage_node,
            as_client_node,
            as_voter_node,
            as_orderer_node,
        })
    }

    /**
     Starts the node and returns a sender to send user commands if the node has
    the client role
     **/
    pub async fn run(&mut self) -> Option<LedgeraClientRunOutput<PKI, Sess, LAT>> {
        if let Some(storage_node_state) = &mut self.as_storage_node {
            match storage_node_state.run().await {
                Ok(_) => {
                    log::info!("storage node thread started for node");
                }
                Err(e) => {
                    log::error!("could not run as storage node : {:?}", e);
                }
            }
        }
        if let Some(voter_node_state) = &mut self.as_voter_node {
            match voter_node_state.run().await {
                Ok(_) => {
                    log::info!("voter node thread started for node");
                }
                Err(e) => {
                    log::error!("could not run as voter node : {:?}", e);
                }
            }
        }
        if let Some(orderer_node_state) = &mut self.as_orderer_node {
            match orderer_node_state.run().await {
                Ok(_) => {
                    log::info!("secure logger node thread started for node");
                }
                Err(e) => {
                    log::error!("could not run as orderer node : {:?}", e);
                }
            }
        }

        if let Some(client_node_state) = &mut self.as_client_node {
            match client_node_state.run().await {
                Ok(client_run_output) => Some(client_run_output),
                Err(e) => {
                    log::error!("could not run as client node : {:?}", e);
                    None
                }
            }
        } else {
            None
        }
    }
}
