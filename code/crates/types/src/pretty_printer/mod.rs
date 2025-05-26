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

use ledgera_pki::message::SignatureEntry;
use ledgera_pki::quorum::QuorumOfSignatures;

fn add_tab_to_lines(number_of_tabs: usize, s: &str) -> String {
    s.lines()
        .map(|line| format!("{}{}", "  ".repeat(number_of_tabs), line))
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn pretty_print_signature(s: &SignatureEntry) -> String {
    format!(
        "Signature : {{\n  signer_public_key : {},\n  signature : {}{}\n}}",
        hex::encode(s.serialized_signing_public_key),
        hex::encode(s.serializable_signature.get_part1()),
        hex::encode(s.serializable_signature.get_part2())
    )
}

pub fn pretty_print_votes_quorum_signatures(
    number_of_tabs: usize,
    q: &QuorumOfSignatures,
) -> String {
    let signatures: Vec<String> = q
        .signatures
        .iter()
        .map(|s| add_tab_to_lines(number_of_tabs, &pretty_print_signature(s)))
        .collect();
    signatures.join(",\n")
}
