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

use ledgera_pki::manager::PKI_SERIALIZED_PUBLIC_KEY_LENGTH;

/**
 * This identifier allows one to distinguish several computation instances:
 * - that use the same computation specification
 * - and that are sent by the same client
 *
 * An honest client will provide different "client_given_ids" for different instances
 * We ensure the identity of the client by comparing this internal public key to the signature of the Rfun message that contains it
 **/
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComputationInstanceInternalIdentifier {
    pub serialized_public_key: [u8; PKI_SERIALIZED_PUBLIC_KEY_LENGTH],
    pub client_given_id: u32,
}

impl ComputationInstanceInternalIdentifier {
    pub fn new(serialized_public_key: [u8; 32], client_given_id: u32) -> Self {
        Self {
            serialized_public_key,
            client_given_id,
        }
    }
}

unsafe impl Send for ComputationInstanceInternalIdentifier {}
