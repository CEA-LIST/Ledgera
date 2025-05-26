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

use ledgera_pki::backends::dalek_backend::implem::DefaultPublicKeyInfrastructureBackend;
use ledgera_pki::manager::PublicKeyInfrastructure;
use ledgera_util_deployment::deployment::{write_participants, write_private_key};
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let number_of_nodes: u32 = {
        let args: Vec<String> = std::env::args().collect();
        if args.len() != 2 {
            eprintln!("Usage: {} <number of nodes>", args[0]);
            std::process::exit(1);
        }
        args[1].parse().unwrap()
    };

    {
        let path = Path::new("./testnet/");
        if path.exists() {
            std::fs::remove_dir_all(path).unwrap();
        }
    }

    let mut private_keys = vec![];
    let mut known_participants = vec![];
    for _ in 0..number_of_nodes {
        let private_key = DefaultPublicKeyInfrastructureBackend::generate_signing_key();
        let public_key =
            DefaultPublicKeyInfrastructureBackend::get_verifying_key_from_signing_key(&private_key);
        private_keys.push(private_key);
        known_participants.push(public_key);
    }

    let know_participants_file_content = write_participants(&known_participants);

    for (node_index, private_key) in private_keys.into_iter().enumerate() {
        {
            let file_path = format!("./testnet/node{}", node_index + 1);
            let path = Path::new(&file_path);
            if !path.exists() {
                std::fs::create_dir_all(path).unwrap();
            }
        }
        {
            let private_key_as_str = write_private_key(&private_key);
            let mut file =
                File::create(format!("./testnet/node{}/private_key.txt", node_index + 1)).unwrap();
            file.write_all(private_key_as_str.as_bytes()).unwrap();
        }
        {
            let mut file = File::create(format!(
                "./testnet/node{}/known_participants.txt",
                node_index + 1
            ))
            .unwrap();
            file.write_all(know_participants_file_content.as_bytes())
                .unwrap();
        }
    }
}
