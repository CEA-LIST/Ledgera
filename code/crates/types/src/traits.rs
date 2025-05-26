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

use ledgera_pki::manager::{KnownParticipantsMap, PublicKeyInfrastructure};

use crate::error::LedgeraInternalApiError;

/// a trait gathering all the requirements for abstract types that are
/// included in ledgera messages
pub trait LedgeraCommunicatableItem:
    std::fmt::Debug
    + Send
    + Sync
    + PartialEq
    + Eq
    + Clone
    + serde::Serialize
    + for<'a> serde::Deserialize<'a>
    + 'static
{
}

/// Ledgera messages that are published on a subscription topic must implement this trait
pub trait LedgeraPublishableMessage:
    std::fmt::Debug + Send + Sync + serde::Serialize + for<'a> serde::Deserialize<'a> + 'static
{
    fn get_msg_type() -> &'static str;
}

pub trait LedgeraQuorumContainingMessage: LedgeraPublishableMessage {
    fn verify_vote_quorums<PKI: PublicKeyInfrastructure>(
        &self,
        known_participants: &KnownParticipantsMap<PKI::VerifyingKey>,
        threshold: u32,
    ) -> Result<(), LedgeraInternalApiError>;
}
