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

use ledgera_node_client::comms::feedback_from_core_client::ValidatedComputationInstance;
use ledgera_pki::manager::SerdeSerializable64BitsSignature;
use ledgera_types::app_template::spec::LedgeraAtomicOperationSpecification;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::messages::nres::LedgeraComputationResultNotification;
use ledgera_types::proofs::proof_of_declaration::ProofOfFunctionDeclaration;
use ledgera_types::proofs::proof_of_integrity::ProofOfOperationIntegrity;
use ledgera_types::proofs::proof_of_unknown_arguments_assignment_verification::ProofOfUnknownArgumentsAssignmentVerification;
use ledgera_types::votes::vout::LedgeraFunctionInstanceOutputKind;

/// everything that the client currently knows about a specific computation instance
pub struct LedgeraFunctionInstanceKnowledgeRepresentation<LAT: LedgeraApplicationTemplate> {
    // the computation instance's id (corresponds to the signature of the initial Rfun message)
    pub id: SerdeSerializable64BitsSignature,

    pub proofs_of_declaration: HashSet<ProofOfFunctionDeclaration>,

    // a client knows the spec of a computation instance if:
    // - either it is the client that has emitted the corresponding Rfun
    // - or it has received a dedicated Ccomp message from another client
    pub spec: Option<LedgeraAtomicOperationSpecification<LAT>>,

    // a client knows the agreed upon unknowns if it has delivered the corresponding "Tins" transaction
    pub agreed_upon_unknowns: Option<ProofOfUnknownArgumentsAssignmentVerification>,

    // a client knows the digest of the result of a computation if:
    // - either it has received one of the Nres notifications
    // - or it has delivered the unique Tout transaction
    pub result_kind: Option<LedgeraFunctionInstanceOutputKind>,

    // keeps track of proofs of validity of the computation result
    // a client may receive different proofs of integrity for the same result:
    // - either via receiving Nres notification
    // - or the unique delivered Tout transaction
    pub proofs_of_result_integrity: HashSet<ProofOfOperationIntegrity>,
}

impl<LAT: LedgeraApplicationTemplate> Clone
    for LedgeraFunctionInstanceKnowledgeRepresentation<LAT>
{
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            proofs_of_declaration: self.proofs_of_declaration.clone(),
            spec: self.spec.clone(),
            agreed_upon_unknowns: self.agreed_upon_unknowns.clone(),
            result_kind: self.result_kind.clone(),
            proofs_of_result_integrity: self.proofs_of_result_integrity.clone(),
        }
    }
}

impl<LAT: LedgeraApplicationTemplate> PartialEq
    for LedgeraFunctionInstanceKnowledgeRepresentation<LAT>
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.spec == other.spec
            && self.agreed_upon_unknowns == other.agreed_upon_unknowns
            && self.result_kind == other.result_kind
            && self.proofs_of_result_integrity == other.proofs_of_result_integrity
            && self.proofs_of_declaration == other.proofs_of_declaration
    }
}

impl<LAT: LedgeraApplicationTemplate> Eq for LedgeraFunctionInstanceKnowledgeRepresentation<LAT> {}

impl<LAT: LedgeraApplicationTemplate> LedgeraFunctionInstanceKnowledgeRepresentation<LAT> {
    pub fn new(id: SerdeSerializable64BitsSignature) -> Self {
        Self {
            id,
            spec: None,
            agreed_upon_unknowns: None,
            result_kind: None,
            proofs_of_result_integrity: HashSet::new(),
            proofs_of_declaration: HashSet::new(),
        }
    }

    pub fn process_validated_function_instance(
        &mut self,
        validated_function_instance: ValidatedComputationInstance<LAT>,
    ) {
        self.spec = Some(validated_function_instance.rfun.spec);
        self.proofs_of_declaration
            .insert(validated_function_instance.delivered_tcomp);
    }

    pub fn process_nout(&mut self, nout: LedgeraComputationResultNotification<LAT::Data>) {
        assert_eq!(self.id, nout.poi.v.function_instance_identifier);
        if self.result_kind.is_none() {
            self.result_kind = Some(nout.poi.v.result_kind.clone());
        }
        self.proofs_of_result_integrity.insert(nout.poi);
    }

    pub fn process_tins(&mut self, tins: ProofOfUnknownArgumentsAssignmentVerification) {
        assert_eq!(self.id, tins.v.function_instance_identifier);
        if self.agreed_upon_unknowns.is_some() {
            panic!("cannot receive duplicate Tins")
        }
        self.agreed_upon_unknowns = Some(tins);
    }

    pub fn process_poi(&mut self, poi: ProofOfOperationIntegrity) {
        assert_eq!(self.id, poi.v.function_instance_identifier);
        self.result_kind = Some(poi.v.result_kind.clone());
        self.proofs_of_result_integrity.insert(poi);
    }
}
