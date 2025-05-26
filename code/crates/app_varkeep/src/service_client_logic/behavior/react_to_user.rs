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

use ledgera_comms::comm_session::PubSubNetwork;
use ledgera_pki::manager::PublicKeyInfrastructure;
use ledgera_types::app_template::input::LedgeraInputArgument;
use ledgera_types::app_template::operation::LedgeraAtomicOperation;
use ledgera_types::app_template::spec::LedgeraAtomicOperationSpecification;

use crate::lat_binding::{VarkeepData, VarkeepTag};
use crate::service_client_logic::role::LEDGERA_VARKEEP_SERVICE_CLIENT_ROLE;
use crate::service_client_logic::service_msgs::messages::LedgeraVarkeepServicePublishLocVarMsg;
use crate::service_client_logic::service_msgs::topics::VarkeepServicesDedicatedTopics;
use crate::service_client_logic::user_reqs::HighLevelVarkeepUserRequests;

use super::LedgeraServiceClientBehavior;

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork> LedgeraServiceClientBehavior<PKI, Sess> {
    pub async fn react_to_service_user_req(
        &mut self,
        service_user_req: HighLevelVarkeepUserRequests,
    ) {
        match service_user_req {
            HighLevelVarkeepUserRequests::AssignLocal(varname, varvalue) => {
                let publish_msg = LedgeraVarkeepServicePublishLocVarMsg::new(varname, varvalue);
                let mut comm_sess = self.comm_session.lock().await;
                match comm_sess
                    .serialize_and_publish_on_topic::<LedgeraVarkeepServicePublishLocVarMsg>(
                        &self.comm_params,
                        &VarkeepServicesDedicatedTopics::PublishLocalVariable.get_topic_str("NA"),
                        &publish_msg,
                    )
                    .await
                {
                    Err(e) => {
                        log::warn!(
                            "As {:?} : could not emit PUBLISH LOCAL VAR : {:?}",
                            LEDGERA_VARKEEP_SERVICE_CLIENT_ROLE,
                            e
                        );
                    }
                    Ok(()) => {
                        log::info!(
                            "As {:?} : published local var",
                            LEDGERA_VARKEEP_SERVICE_CLIENT_ROLE,
                        );
                    }
                }
            }
            HighLevelVarkeepUserRequests::AssignGlobal(varname, varvalue) => {
                let core_varname = LedgeraInputArgument::RawValue {
                    is_input_persistent: false,
                    value: VarkeepData::VariableName(varname),
                };
                let core_varvalue = LedgeraInputArgument::RawValue {
                    is_input_persistent: false,
                    value: VarkeepData::VariableValue(varvalue),
                };
                let function_spec = LedgeraAtomicOperationSpecification::new(
                    LedgeraAtomicOperation::TagInputs(VarkeepTag::Assign),
                    None,
                    vec![core_varname, core_varvalue],
                );
                let _ = self
                    .core_client_runtime_io
                    .compute_function(function_spec)
                    .await;
            }
        }
    }
}
