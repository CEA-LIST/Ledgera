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

use ledgera_comms::{comm_session::PubSubNetwork, error::LedgeraCommunicationError};
use ledgera_types::app_template::template::LedgeraApplicationTemplate;

#[derive(PartialEq, Eq, Clone)]
pub enum VoterComputationBehaviorError<Sess: PubSubNetwork, LAT: LedgeraApplicationTemplate> {
    CouldNotEmitVfun(LedgeraCommunicationError<Sess::CommRuntimeError>),
    CouldNotEmitTfun(LedgeraCommunicationError<Sess::CommRuntimeError>),
    // ***
    CouldNotEmitVins(LedgeraCommunicationError<Sess::CommRuntimeError>),
    CouldNotEmitTins(LedgeraCommunicationError<Sess::CommRuntimeError>),
    // ***
    ErrorWhenComputingLocalResult(LAT::RuntimeError),
    TryingComputeOnATagOperation,
    // ***
    CouldNotEmitVout(LedgeraCommunicationError<Sess::CommRuntimeError>),
    CouldNotEmitNout(LedgeraCommunicationError<Sess::CommRuntimeError>),
    CouldNotEmitTout(LedgeraCommunicationError<Sess::CommRuntimeError>),
    // ***
    Panicked,
    // ***
    InvalidRfun,
    TinsDeliveryChannelClosed,
}

impl<Sess: PubSubNetwork, LAT: LedgeraApplicationTemplate> std::fmt::Debug
    for VoterComputationBehaviorError<Sess, LAT>
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VoterComputationBehaviorError::CouldNotEmitVfun(ledgera_communication_error) => {
                write!(
                    f,
                    "CouldNotEmitVfun due to : {:?}",
                    ledgera_communication_error
                )
            }
            VoterComputationBehaviorError::CouldNotEmitTfun(ledgera_communication_error) => {
                write!(
                    f,
                    "CouldNotEmitTfun due to : {:?}",
                    ledgera_communication_error
                )
            }
            VoterComputationBehaviorError::CouldNotEmitVins(ledgera_communication_error) => {
                write!(
                    f,
                    "CouldNotEmitVins due to : {:?}",
                    ledgera_communication_error
                )
            }
            VoterComputationBehaviorError::CouldNotEmitTins(ledgera_communication_error) => {
                write!(
                    f,
                    "CouldNotEmitTins due to : {:?}",
                    ledgera_communication_error
                )
            }
            VoterComputationBehaviorError::ErrorWhenComputingLocalResult(service_runtime_error) => {
                write!(
                    f,
                    "ErrorWhenComputingLocalResult due to : {:?}",
                    service_runtime_error
                )
            }
            VoterComputationBehaviorError::CouldNotEmitVout(ledgera_communication_error) => {
                write!(
                    f,
                    "CouldNotEmitVout due to : {:?}",
                    ledgera_communication_error
                )
            }
            VoterComputationBehaviorError::CouldNotEmitNout(ledgera_communication_error) => {
                write!(
                    f,
                    "CouldNotEmitNout due to : {:?}",
                    ledgera_communication_error
                )
            }
            VoterComputationBehaviorError::CouldNotEmitTout(ledgera_communication_error) => {
                write!(
                    f,
                    "CouldNotEmitTout due to : {:?}",
                    ledgera_communication_error
                )
            }
            VoterComputationBehaviorError::TryingComputeOnATagOperation => {
                write!(f, "TryingComputeOnATagOperation")
            }
            VoterComputationBehaviorError::Panicked => {
                write!(f, "Panicked")
            }
            VoterComputationBehaviorError::InvalidRfun => {
                write!(f, "InvalidRfun")
            }
            VoterComputationBehaviorError::TinsDeliveryChannelClosed => {
                write!(f, "TinsDeliveryChannelClosed")
            }
        }
    }
}
