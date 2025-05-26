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

use ledgera_pki::manager::SerdeSerializable64BitsSignature;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::messages::nres::LedgeraComputationResultNotification;
use ledgera_types::proofs::proof_of_declaration::ProofOfFunctionDeclaration;
use ledgera_types::proofs::proof_of_integrity::ProofOfOperationIntegrity;
use ledgera_types::proofs::proof_of_storage::ProofOfShipmentToStorage;
use ledgera_types::proofs::proof_of_unknown_arguments_assignment_verification::ProofOfUnknownArgumentsAssignmentVerification;
use ledgera_types::requests::rfun::LedgeraRequestFunctionInstanceProposal;

/*
The core client receives and validates:
- Rfun messages,
- Nout messages,
- Delivered Transactions
These messages may be forwarded to the application that uses the core client as an interface
to update its internal state
*/
#[derive(Clone)]
pub enum ValidatedCoreFeedbackMessage<LAT: LedgeraApplicationTemplate> {
    ValidatedComputationInstance(ValidatedComputationInstance<LAT>),
    Nout(LedgeraComputationResultNotification<LAT::Data>),
    DeliveredTsto(ProofOfShipmentToStorage),
    DeliveredTins(ProofOfUnknownArgumentsAssignmentVerification),
    DeliveredTout(ProofOfOperationIntegrity),
}

#[derive(Clone)]
pub struct ValidatedComputationInstance<LAT: LedgeraApplicationTemplate> {
    pub rfun_sig: SerdeSerializable64BitsSignature,
    pub rfun: LedgeraRequestFunctionInstanceProposal<LAT>,
    pub delivered_tcomp: ProofOfFunctionDeclaration,
}

impl<LAT: LedgeraApplicationTemplate> ValidatedComputationInstance<LAT> {
    pub fn new(
        rfun_sig: SerdeSerializable64BitsSignature,
        rfun: LedgeraRequestFunctionInstanceProposal<LAT>,
        delivered_tcomp: ProofOfFunctionDeclaration,
    ) -> Self {
        Self {
            rfun_sig,
            rfun,
            delivered_tcomp,
        }
    }
}
