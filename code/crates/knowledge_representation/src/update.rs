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

use ledgera_node_client::comms::feedback_from_core_client::ValidatedCoreFeedbackMessage;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::{app_template::input::LedgeraInputArgument, digest::LedgeraDigest};

use crate::{
    know::LedgeraKnowledgeRepresentation, per_data::LedgeraDataValueKnowledgeRepresentation,
    per_function_instance::LedgeraFunctionInstanceKnowledgeRepresentation,
};

impl<LAT: LedgeraApplicationTemplate> LedgeraKnowledgeRepresentation<LAT> {
    /**
     * Updates the knowledge using data from the application layer
     * **/
    pub fn update_per_application_input_data(
        &mut self,
        data_digest: LedgeraDigest,
        data_value: LAT::Data,
    ) {
        let per_data = self
            .per_data_value
            .entry(data_digest.clone())
            .or_insert(LedgeraDataValueKnowledgeRepresentation::new(data_digest));
        per_data.process_value(data_value);
    }

    /**
     * Updates the knowledge using feeback from Ledgera Core
     * **/
    pub fn update_per_feedback_from_core(&mut self, feedback: ValidatedCoreFeedbackMessage<LAT>) {
        match feedback {
            ValidatedCoreFeedbackMessage::ValidatedComputationInstance(
                validated_computation_instance,
            ) => {
                self.function_instances_order_in_log
                    .push(validated_computation_instance.rfun_sig.clone());
                let per_instance = self
                    .per_function_instance
                    .entry(validated_computation_instance.rfun_sig.clone())
                    .or_insert(LedgeraFunctionInstanceKnowledgeRepresentation::new(
                        validated_computation_instance.rfun_sig.clone(),
                    ));
                for val in &validated_computation_instance.rfun.spec.arguments {
                    match val {
                        LedgeraInputArgument::RawValue {
                            is_input_persistent,
                            value,
                        } if *is_input_persistent => {
                            let data_digest = LedgeraDigest::from_serializable(&value).unwrap();
                            let per_data =
                                self.per_data_value.entry(data_digest.clone()).or_insert(
                                    LedgeraDataValueKnowledgeRepresentation::new(data_digest),
                                );
                            per_data.process_value(value.clone());
                        }
                        LedgeraInputArgument::ReferenceToStorage(pos) => {
                            let per_data = self
                                .per_data_value
                                .entry(pos.v.data_digest.clone())
                                .or_insert(LedgeraDataValueKnowledgeRepresentation::new(
                                    pos.v.data_digest.clone(),
                                ));
                            per_data.process_pos(pos.clone());
                        }
                        _ => {}
                    }
                }
                per_instance.process_validated_function_instance(validated_computation_instance);
            }
            ValidatedCoreFeedbackMessage::Nout(nout) => {
                let per_instance = self
                    .per_function_instance
                    .entry(nout.poi.v.function_instance_identifier.clone())
                    .or_insert(LedgeraFunctionInstanceKnowledgeRepresentation::new(
                        nout.poi.v.function_instance_identifier.clone(),
                    ));
                let output_digest = LedgeraDigest::from_serializable(&nout.result_value).unwrap();
                if let Some(per_data) = self.per_data_value.get_mut(&output_digest) {
                    per_data.data_value = Some(nout.result_value.clone());
                }
                per_instance.process_nout(nout);
            }
            ValidatedCoreFeedbackMessage::DeliveredTsto(pos) => {
                let per_data = self
                    .per_data_value
                    .entry(pos.v.data_digest.clone())
                    .or_insert(LedgeraDataValueKnowledgeRepresentation::new(
                        pos.v.data_digest.clone(),
                    ));
                per_data.process_pos(pos);
            }
            ValidatedCoreFeedbackMessage::DeliveredTins(anchored_pouav) => {
                let per_instance = self
                    .per_function_instance
                    .entry(anchored_pouav.v.function_instance_identifier.clone())
                    .or_insert(LedgeraFunctionInstanceKnowledgeRepresentation::new(
                        anchored_pouav.v.function_instance_identifier.clone(),
                    ));
                per_instance.process_tins(anchored_pouav);
            }
            ValidatedCoreFeedbackMessage::DeliveredTout(poi) => {
                let per_instance = self
                    .per_function_instance
                    .entry(poi.v.function_instance_identifier.clone())
                    .or_insert(LedgeraFunctionInstanceKnowledgeRepresentation::new(
                        poi.v.function_instance_identifier.clone(),
                    ));
                per_instance.process_poi(poi);
            }
        }
    }
}
