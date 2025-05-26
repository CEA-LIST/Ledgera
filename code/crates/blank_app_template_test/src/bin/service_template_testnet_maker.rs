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

use std::fs::File;
use std::io::Write;
use std::path::Path;

use ledgera_pki::backends::dalek_backend::implem::DefaultPublicKeyInfrastructureBackend;
use ledgera_pki::manager::PublicKeyInfrastructure;
use ledgera_util_deployment::deployment::{write_participants, write_private_key};

const TESTNET_DIR: &str = "./service_template_testnet";

fn main() {
    let (number_of_nodes, number_of_service_clients) = {
        let args: Vec<String> = std::env::args().collect();
        if args.len() != 3 {
            eprintln!(
                "Usage: {} <number of nodes> <number of service clients>",
                args[0]
            );
            std::process::exit(1);
        }
        let number_of_nodes: u32 = args[1].parse().unwrap();
        let number_of_service_clients: u32 = args[2].parse().unwrap();
        (number_of_nodes, number_of_service_clients)
    };

    {
        let path = Path::new(TESTNET_DIR);
        if path.exists() {
            std::fs::remove_dir_all(path).unwrap();
        }
    }

    let mut private_keys = vec![];
    let mut known_participants = vec![];
    let mut known_service_clients = vec![];
    for node_id in 0..number_of_nodes {
        let private_key = DefaultPublicKeyInfrastructureBackend::generate_signing_key();
        let public_key =
            DefaultPublicKeyInfrastructureBackend::get_verifying_key_from_signing_key(&private_key);
        private_keys.push(private_key);
        known_participants.push(public_key);
        if node_id < number_of_service_clients {
            known_service_clients.push(public_key);
        }
    }

    let known_participants_file_content = write_participants(&known_participants);
    let service_clients_file_content = write_participants(&known_service_clients);

    for (node_index, private_key) in private_keys.into_iter().enumerate() {
        let node_dir = format!("{}/node{}", TESTNET_DIR, node_index + 1);
        std::fs::create_dir_all(&node_dir).unwrap();

        let mut f = File::create(format!("{}/private_key.txt", node_dir)).unwrap();
        f.write_all(write_private_key(&private_key).as_bytes())
            .unwrap();

        let mut f = File::create(format!("{}/known_participants.txt", node_dir)).unwrap();
        f.write_all(known_participants_file_content.as_bytes())
            .unwrap();

        let mut f = File::create(format!("{}/service_clients.txt", node_dir)).unwrap();
        f.write_all(service_clients_file_content.as_bytes())
            .unwrap();
    }
}
