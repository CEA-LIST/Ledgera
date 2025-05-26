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

use std::collections::BTreeMap;

use ledgera_pki::manager::SerdeSerializable64BitsSignature;

use crate::traits::LedgeraPublishableMessage;

use crate::digest::LedgeraDigest;

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize, Hash)]
pub struct UnknownArgumentsAggreementReference {
    pub digest_of_delivered_tins: LedgeraDigest,
    pub digests_of_unknowns: BTreeMap<u32, LedgeraDigest>,
}

impl UnknownArgumentsAggreementReference {
    pub fn new(
        digest_of_delivered_tins: LedgeraDigest,
        digests_of_unknowns: BTreeMap<u32, LedgeraDigest>,
    ) -> Self {
        Self {
            digest_of_delivered_tins,
            digests_of_unknowns,
        }
    }
}

/**
 * Given a function instance initiated by a "Rfun" request:
 * This "LedgeraFunctionInstanceOutputKind" object is a lightweight representation of the output produced as a result of its execution.
 * It is this object that is used in "Vout" votes to produce a Proof Of Integrity.
 *
 * The "LedgeraFunctionInstanceOutputKind" is an enum with 2 possible variants:
 * - if the function computes an output, we use the "ComputedOutput" variant which includes:
 *   + the digest of the computed output
 *   + and a copy of the output persistence flag from the "Rfun",
 *     so that the PoI that will eventually be created (which includes the "Vout", which itself includes this present object)
 *     can justify storage of the value in the distributed storage.
 * - if the function is "the identity", we use the "TaggedInputs" variant.
 *   Indeed, with "the identity function", only its inputs can be flagged persistent and there is no need to compute an output
 *   so we use a dedicated enum variant.
 * **/
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize, Hash)]
pub enum LedgeraFunctionInstanceOutputKind {
    ComputedOutput {
        is_output_persistent: bool,
        output_digest: LedgeraDigest,
    },
    TaggedInputs,
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize, Hash)]
pub struct LedgeraVoteFunctionOutput {
    /// identifies the operation instance via the signature of the initial 'Rfun' message that requested its declaration and execution
    pub function_instance_identifier: SerdeSerializable64BitsSignature,
    /// identifies (if it exists) the digest of the "Tins" transaction in which the unknown arguments of the computation instance were agreed-upon
    pub unknowns_agreement_ref: Option<LedgeraDigest>,
    /// description of the result of the operation
    pub result_kind: LedgeraFunctionInstanceOutputKind,
}

impl LedgeraVoteFunctionOutput {
    pub fn new(
        function_instance_identifier: SerdeSerializable64BitsSignature,
        unknowns_agreement_ref: Option<LedgeraDigest>,
        result_kind: LedgeraFunctionInstanceOutputKind,
    ) -> Self {
        Self {
            function_instance_identifier,
            unknowns_agreement_ref,
            result_kind,
        }
    }
}

impl LedgeraPublishableMessage for LedgeraVoteFunctionOutput {
    fn get_msg_type() -> &'static str {
        "Vout"
    }
}
