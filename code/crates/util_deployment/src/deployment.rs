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

use std::collections::{BTreeSet, HashMap};

use ledgera_pki::manager::KnownParticipantsMap;

pub fn write_private_key(private_key: &ed25519_dalek::SigningKey) -> String {
    let private_key_as_bytes = private_key.to_bytes();
    hex::encode(private_key_as_bytes)
}

pub fn read_private_key(file_content: String) -> ed25519_dalek::SigningKey {
    let decoded_vec: Vec<u8> = hex::decode(file_content.as_bytes()).unwrap();
    let decoded_arr: [u8; 32] = decoded_vec.try_into().unwrap();
    ed25519_dalek::SigningKey::from_bytes(&decoded_arr)
}

pub fn write_participants(known_participants: &[ed25519_dalek::VerifyingKey]) -> String {
    known_participants
        .iter()
        .map(|pub_key| hex::encode(pub_key.to_bytes()))
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn read_known_participants(
    file_content: String,
) -> KnownParticipantsMap<ed25519_dalek::VerifyingKey> {
    let mut known_participants = HashMap::new();
    for key_hex in file_content.split("\n") {
        let decoded_vec = hex::decode(key_hex.as_bytes()).unwrap();
        let decoded_arr: [u8; 32] = decoded_vec.try_into().unwrap();
        let vk = ed25519_dalek::VerifyingKey::from_bytes(&decoded_arr).unwrap();
        known_participants.insert(decoded_arr, vk);
    }
    known_participants
}

pub fn read_service_clients(file_content: String) -> BTreeSet<String> {
    let mut client_names = BTreeSet::new();
    for key_hex in file_content.split("\n") {
        client_names.insert(key_hex.to_string());
    }
    client_names
}
