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

use crate::backends::dalek_backend::implem::DefaultPublicKeyInfrastructureBackend;

use crate::tests::*;

#[test]
fn test_message_signing_and_verification() {
    tool_test_message_signing_and_verification::<DefaultPublicKeyInfrastructureBackend>()
}

#[test]
fn test_serialize_deserialize_verifying_key() {
    tool_test_serialize_deserialize_verifying_key::<DefaultPublicKeyInfrastructureBackend>()
}

#[test]
fn test_serialize_deserialize_signature() {
    tool_test_serialize_deserialize_signature::<DefaultPublicKeyInfrastructureBackend>();
}

#[test]
fn test_deserialize_nonsense_as_verifying_key() {
    tool_test_deserialize_nonsense_as_verifying_key::<DefaultPublicKeyInfrastructureBackend>();
}
