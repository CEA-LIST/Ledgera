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

pub mod proof_of_declaration;
pub mod proof_of_integrity;
pub mod proof_of_storage;
pub mod proof_of_unknown_arguments_assignment_verification;

use ledgera_pki::manager::KnownParticipantsMap;
use ledgera_pki::quorum::QuorumOfSignatures;

use crate::error::{LedgeraInternalApiError, LedgeraInternalApiErrorContext};

pub(crate) fn verify_quorum_agreement<PKI, V>(
    vote: &V,
    quorum: &QuorumOfSignatures,
    known_participants: &KnownParticipantsMap<PKI::VerifyingKey>,
    threshold: u32,
    context: &'static str,
) -> Result<(), LedgeraInternalApiError>
where
    PKI: ledgera_pki::manager::PublicKeyInfrastructure,
    V: serde::Serialize,
{
    let serialized_vote =
        bincode::serialize(vote).map_err(|_| LedgeraInternalApiError::CannotSerializeMessage)?;
    if serialized_vote != quorum.agreed_upon_value {
        return Err(LedgeraInternalApiError::InContext(
            LedgeraInternalApiErrorContext::WhenVerifying(context),
            Box::new(LedgeraInternalApiError::QuorumAgreedUponValueDoesNotMatchContext),
        ));
    }
    quorum
        .is_a_valid_quorum::<PKI>(known_participants, threshold)
        .map_err(|e| {
            LedgeraInternalApiError::InContext(
                LedgeraInternalApiErrorContext::WhenVerifying(context),
                Box::new(LedgeraInternalApiError::PkiError(e)),
            )
        })
}
