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
use ledgera_node_client::comms::feedback_from_core_client::ValidatedCoreFeedbackMessage;
use ledgera_pki::manager::PublicKeyInfrastructure;
use ledgera_types::app_template::input::LedgeraInputArgument;

use crate::lat_binding::{LedgeraVarkeepService, VarkeepData};

use super::LedgeraServiceClientBehavior;

impl<PKI: PublicKeyInfrastructure, Sess: PubSubNetwork> LedgeraServiceClientBehavior<PKI, Sess> {
    pub async fn react_to_validated_core_msg(
        &mut self,
        validated_core_msg: ValidatedCoreFeedbackMessage<LedgeraVarkeepService>,
    ) {
        match validated_core_msg {
            ValidatedCoreFeedbackMessage::ValidatedComputationInstance(comp_instance) => {
                let _comp_sig = comp_instance.rfun_sig;
                let varname = comp_instance.rfun.spec.arguments.first().unwrap();
                let varvalue = comp_instance.rfun.spec.arguments.get(1).unwrap();
                match (varname, varvalue) {
                    (
                        LedgeraInputArgument::RawValue {
                            is_input_persistent: _,
                            value: vn,
                        },
                        LedgeraInputArgument::RawValue {
                            is_input_persistent: _,
                            value: vv,
                        },
                    ) => match (vn, vv) {
                        (VarkeepData::VariableName(rvn), VarkeepData::VariableValue(rvv)) => {
                            let _ = self
                                .to_ui_feed
                                .send((format!("core.{}", rvn), rvv.to_string()))
                                .await;
                        }
                        _ => {
                            // ignore it
                        }
                    },
                    _ => {
                        // ignore it
                    }
                }
            }
            _ => {
                // ignore it
            }
        }
    }
}
