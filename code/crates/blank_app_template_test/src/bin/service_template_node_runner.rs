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

use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::Duration;

use ledgera_blank_app_template::lat_binding::LedgeraServiceTemplate;
use ledgera_blank_app_template::service_client_logic::service_client_state::LedgeraServiceClientState;
use ledgera_blank_app_template_tui::setup::{ledgera_service_tui_setup, ledgera_tui_log_setup};
use ledgera_comms::comm_api::{
    LedgeraInternalCommunicationInterface, LedgeraInternalCommunicationParameters,
};
use ledgera_comms_zenoh::backend::ZenohBackend;
use ledgera_core_logic::roles::LedgeraCoreRoles;
use ledgera_node::node::LedgerNodeState;
use ledgera_pki::backends::dalek_backend::implem::DefaultPublicKeyInfrastructureBackend;
use ledgera_util_deployment::deployment::{
    read_known_participants, read_private_key, read_service_clients,
};
use ledgera_util_deployment::zenoh_config::{PLACEHOLDER_NAME, RAW_ZENOH_CONFIG};
use log::LevelFilter;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let (node_name, roles, pki_folder_path) = {
        let args: Vec<String> = std::env::args().collect();
        if args.len() != 4 {
            eprintln!(
                "Usage: {} <node name> <roles abbrs. ('c' client, 's' storage, 'v' voter, 'l' log)> <pki folder path>",
                args[0]
            );
            std::process::exit(1);
        }
        let roles_abbrs: String = args[2].to_owned();
        let mut roles = HashSet::new();
        if roles_abbrs.contains("c") {
            roles.insert(LedgeraCoreRoles::Client);
        }
        if roles_abbrs.contains("s") {
            roles.insert(LedgeraCoreRoles::PersistentStorage);
        }
        if roles_abbrs.contains("v") {
            roles.insert(LedgeraCoreRoles::VoterComputer);
        }
        if roles_abbrs.contains("l") {
            roles.insert(LedgeraCoreRoles::SecureLogger);
        }
        let folder_path = args[3].to_owned();
        (
            args[1].to_owned(),
            roles,
            fs::canonicalize(folder_path).unwrap(),
        )
    };

    if !roles.contains(&LedgeraCoreRoles::Client) {
        if std::env::var("RUST_LOG").is_err() {
            unsafe {
                std::env::set_var("RUST_LOG", "info");
            }
        }
        env_logger::Builder::new()
            .format(|buf, record| writeln!(buf, "[{}] - {}", record.level(), record.args()))
            .filter_level(LevelFilter::Info)
            .init();
    } else {
        ledgera_tui_log_setup();
    }

    log::warn!("making Zenoh configuration for node {:}", node_name);
    let config = {
        let conf = RAW_ZENOH_CONFIG.to_owned();
        let _ = conf.replace(PLACEHOLDER_NAME, &node_name);
        zenoh::Config::from_json5(&conf)
            .expect("Failed to load the default Zenoh configuration file")
    };

    log::warn!(
        "loading private key from current folder for node {:}",
        node_name
    );
    let private_key = {
        let private_key_file_path = pki_folder_path.join("private_key.txt");
        let mut contents = String::new();
        File::open(private_key_file_path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        read_private_key(contents)
    };
    log::warn!(
        "loading known participants from current folder for node {:}",
        node_name
    );
    let known_participants = {
        let known_participants_file_path = pki_folder_path.join("known_participants.txt");
        let mut contents = String::new();
        File::open(known_participants_file_path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        read_known_participants(contents)
    };

    let all_service_clients_names = {
        let clients_file_path = pki_folder_path.join("service_clients.txt");
        let mut contents = String::new();
        File::open(clients_file_path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        read_service_clients(contents)
    };

    let threshold = (known_participants.len() - 1) / 3;
    log::info!(
        "As there are {} known participants, we consider a threshold 'f' of {}",
        known_participants.len(),
        threshold
    );

    let comm_param = Arc::new(LedgeraInternalCommunicationParameters::new(
        threshold as u32,
        Arc::new(private_key),
        Arc::new(known_participants),
    ));
    let service = Arc::new(LedgeraServiceTemplate);
    let comm_api = LedgeraInternalCommunicationInterface::<
        DefaultPublicKeyInfrastructureBackend,
        ZenohBackend,
    >::from_config(config)
    .await
    .unwrap();
    let comm_api_ref = Arc::new(Mutex::new(comm_api));
    let mut ledgera_node_state: LedgerNodeState<
        DefaultPublicKeyInfrastructureBackend,
        ZenohBackend,
        LedgeraServiceTemplate,
    > = LedgerNodeState::new(
        comm_param.clone(),
        service.clone(),
        comm_api_ref.clone(),
        roles,
    )
    .await
    .unwrap();

    match ledgera_node_state.run().await {
        None => loop {
            tokio::time::sleep(Duration::from_millis(10)).await;
        },
        Some(core_client_run) => {
            let mut service_node_state =
                LedgeraServiceClientState::new(comm_api_ref, comm_param, service);
            let service_node_runtime_io = service_node_state
                .run(
                    core_client_run.core_runtime,
                    core_client_run.to_app_stream_of_validated_core_msgs,
                )
                .await
                .unwrap();
            ledgera_service_tui_setup(
                node_name,
                service_node_runtime_io,
                all_service_clients_names,
            )
            .await;
        }
    }
}
