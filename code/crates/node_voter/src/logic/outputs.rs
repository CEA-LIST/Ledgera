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

use std::{collections::HashMap, marker::PhantomData};

use ledgera_pki::manager::PKI_SERIALIZED_PUBLIC_KEY_LENGTH;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::{
    digest::LedgeraDigest,
    proofs::{
        proof_of_declaration::ProofOfFunctionDeclaration,
        proof_of_integrity::ProofOfOperationIntegrity,
    },
};

use ledgera_types::app_template::spec::LedgeraAtomicOperationSpecification;

/**
 * In phase 1, we receive the initial 'Mop' ang gives the corresponding operation "execute access"
 * The output of this phase consists in:
 * - the specification of the operation
 * - the quorum of 'Vop' votes constituting the proof of declaration
 * - and additional information about the client that submitted the operation request so that we may later notify it of intermediate results
 * **/
pub struct ComputationInstancePhase1Result<LAT: LedgeraApplicationTemplate> {
    pub op_spec: LedgeraAtomicOperationSpecification<LAT>,
    pub pod: ProofOfFunctionDeclaration,
    pub sender: [u8; PKI_SERIALIZED_PUBLIC_KEY_LENGTH],
}

impl<LAT: LedgeraApplicationTemplate> ComputationInstancePhase1Result<LAT> {
    pub fn new(
        op_spec: LedgeraAtomicOperationSpecification<LAT>,
        pod: ProofOfFunctionDeclaration,
        sender: [u8; PKI_SERIALIZED_PUBLIC_KEY_LENGTH],
    ) -> Self {
        Self {
            op_spec,
            pod,
            sender,
        }
    }
}

/**
 * Phase 2's goal is to collect all required inputs for the realization of the operation.
 * If the operation is a Tag ("identity" that does nothing with the input) and if there are no unknowns to agree on, Phase 2 does nothing.
 * If there are no unknowns but the operation is a non-trivial computation, Phase 2 retrieves all relevent inputs from storage.
 * If there are unknowns (whether or not the operation is a Tag), Phase 2 performs the core-set agreement
 * **/
pub struct ComputationInstancePhase2Result<LAT: LedgeraApplicationTemplate> {
    pub know_arguments_values: HashMap<usize, LAT::Data>,
    pub unknow_arguments_values: Option<HashMap<usize, LAT::Data>>,
    // if the operation has unknowns, we need to deliver a "Tins" during Phase 2
    pub tins_digest: Option<LedgeraDigest>,
    pub phantom: PhantomData<LAT>,
}

impl<LAT: LedgeraApplicationTemplate> ComputationInstancePhase2Result<LAT> {
    pub fn new(
        know_arguments_values: HashMap<usize, LAT::Data>,
        unknow_arguments_values: Option<HashMap<usize, LAT::Data>>,
        tins_digest: Option<LedgeraDigest>,
    ) -> Self {
        Self {
            know_arguments_values,
            unknow_arguments_values,
            tins_digest,
            phantom: PhantomData,
        }
    }
}

pub struct ComputationInstancePhase3Result<DataValue> {
    pub poi: ProofOfOperationIntegrity,
    pub result_value: Option<DataValue>,
}

impl<DataValue> ComputationInstancePhase3Result<DataValue> {
    pub fn new(poi: ProofOfOperationIntegrity, result_value: Option<DataValue>) -> Self {
        Self { poi, result_value }
    }
}
