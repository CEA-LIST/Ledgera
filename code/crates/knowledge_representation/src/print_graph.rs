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

use layout::adt::dag::NodeHandle;
use layout::backends::svg::SVGWriter;
use layout::core::base::Orientation;
use layout::core::color::Color;
use layout::core::geometry::Point;
use layout::core::style::*;
use layout::core::utils::save_to_file;
use layout::std_shapes::shapes::*;
use layout::topo::layout::VisualGraph;
use ledgera_pki::manager::SerdeSerializable64BitsSignature;
use ledgera_types::app_template::input::LedgeraInputArgument;
use ledgera_types::app_template::operation::LedgeraAtomicOperation;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::digest::LedgeraDigest;
use ledgera_types::votes::vsto::PersistentDataKind;
use std::collections::HashMap;

use crate::know::LedgeraKnowledgeRepresentation;
use crate::printer::LedgeraComputationItemsPrinter;

pub fn print_current_knowledge_as_graph<
    LAT: LedgeraApplicationTemplate,
    Printer: LedgeraComputationItemsPrinter<LAT>,
>(
    filename_to_print_graph: &str,
    k: &LedgeraKnowledgeRepresentation<LAT>,
    c_monikers: &HashMap<String, SerdeSerializable64BitsSignature>,
    d_monikers: &HashMap<String, LedgeraDigest>,
) {
    let mut data_nodes: HashMap<LedgeraDigest, NodeHandle> = HashMap::new();
    let mut data_reference_nodes: HashMap<
        (SerdeSerializable64BitsSignature, PersistentDataKind),
        NodeHandle,
    > = HashMap::new();
    let mut comp_instances_nodes: HashMap<SerdeSerializable64BitsSignature, NodeHandle> =
        HashMap::new();
    // Create a new graph:
    let mut vg = VisualGraph::new(Orientation::LeftToRight);

    for (comp_moniker, comp_id) in c_monikers.iter() {
        let mut labels = vec![
            format!("func moniker : {}", comp_moniker),
            format!(
                "func id      : {}..",
                comp_id
                    .to_hexadecimal_string()
                    .chars()
                    .take(8)
                    .collect::<String>()
            ),
        ];
        if let Some(instance_k) = k.per_function_instance.get(comp_id) {
            if let Some(instance_spec) = &instance_k.spec {
                match &instance_spec.operation {
                    LedgeraAtomicOperation::TagInputs(tag) => {
                        labels.push(format!(
                            "tag (identity function) : {:}",
                            Printer::print_tag(tag)
                        ));
                    }
                    LedgeraAtomicOperation::ComputeOutput {
                        is_output_persistent,
                        comp,
                    } => {
                        labels.push(format!(
                            "compute function : {:}",
                            Printer::print_computation(comp)
                        ));
                        labels.push(format!("is_output_persistent : {:?}", is_output_persistent));
                    }
                }
                for (input_idx, input_spec) in instance_spec.arguments.iter().enumerate() {
                    match input_spec {
                        LedgeraInputArgument::RawValue {
                            is_input_persistent,
                            value,
                        } => {
                            labels.push(format!(
                                "input@{:} : RawValue[persist:{:?}, value:{:?}]",
                                input_idx,
                                is_input_persistent,
                                Printer::print_value(value)
                            ));
                        }
                        LedgeraInputArgument::ReferenceToStorage(pos) => {
                            labels.push(format!(
                                "input@{:} : StorageRef[pos_fid:{:}.., stored_as:{:}]",
                                input_idx,
                                pos.v
                                    .function_instance_identifier
                                    .to_hexadecimal_string()
                                    .chars()
                                    .take(8)
                                    .collect::<String>(),
                                pos.v.data_kind
                            ));
                        }
                        LedgeraInputArgument::Unknown(pred) => {
                            labels.push(format!(
                                "input@{:} : Unknown[pred:{:}]",
                                input_idx,
                                Printer::print_local_predicate(pred)
                            ));
                        }
                    }
                }
                if let Some(glo_pred) = &instance_spec.global_arguments_predicate {
                    labels.push(format!(
                        "global_predicate :{:}",
                        Printer::print_global_predicate(glo_pred)
                    ));
                }
            }
        }
        let comp_node = make_node(&labels, "#a1f542");
        let comp_moniker_handle = vg.add_node(comp_node);
        comp_instances_nodes.insert(comp_id.clone(), comp_moniker_handle);
    }

    for (data_moniker, data_digest) in d_monikers.iter() {
        let mut labels = vec![
            format!("data moniker : {}", data_moniker),
            format!(
                "data digest  : {}..",
                data_digest
                    .to_hexadecimal_string()
                    .chars()
                    .take(5)
                    .collect::<String>()
            ),
        ];
        if let Some(data_k) = k.per_data_value.get(data_digest) {
            if let Some(data_value) = &data_k.data_value {
                labels.push(format!(
                    "data_value  : {:}",
                    Printer::print_value(data_value)
                ));
            }
            let data_node = make_node(&labels, "#4DE9F7");
            let data_node_handle = vg.add_node(data_node);
            data_nodes.insert(data_digest.clone(), data_node_handle);
            for (_, poss) in data_k.proofs_of_storage.iter() {
                let pos = poss.iter().next().unwrap();
                let tuple = (
                    pos.v.function_instance_identifier.clone(),
                    pos.v.data_kind.clone(),
                );
                data_reference_nodes.entry(tuple).or_insert_with(|| {
                    let mut labels = vec![format!("stored as")];
                    match pos.v.data_kind {
                        PersistentDataKind::Input(x) => labels.push(format!("{}th input", x)),
                        PersistentDataKind::Output => labels.push("output".to_string()),
                    };

                    let reference_node = make_node(&labels, "#f7bc4d");
                    let reference_node_handle = vg.add_node(reference_node);
                    vg.add_edge(
                        Arrow::simple(""),
                        *comp_instances_nodes
                            .get(&pos.v.function_instance_identifier)
                            .unwrap(),
                        reference_node_handle,
                    );
                    vg.add_edge(
                        Arrow::simple(""),
                        reference_node_handle,
                        *data_nodes.get(&pos.v.data_digest).unwrap(),
                    );
                    reference_node_handle
                });
            }
        }
    }

    for comp_id in c_monikers.values() {
        if let Some(instance_k) = k.per_function_instance.get(comp_id) {
            if let Some(instance_spec) = &instance_k.spec {
                // for the inputs from the spec that are references to storage
                for (arg_id, arg) in instance_spec.arguments.iter().enumerate() {
                    if let LedgeraInputArgument::ReferenceToStorage(pos) = arg {
                        let labels =
                            vec![format!("used as {}th input", arg_id), format!("from spec")];
                        let usage_node = make_node(&labels, "#974df7");
                        let usage_node_handle = vg.add_node(usage_node);
                        let tuple = (
                            pos.v.function_instance_identifier.clone(),
                            pos.v.data_kind.clone(),
                        );
                        vg.add_edge(
                            Arrow::simple(""),
                            *data_reference_nodes.get(&tuple).unwrap(),
                            usage_node_handle,
                        );
                        vg.add_edge(
                            Arrow::simple(""),
                            usage_node_handle,
                            *comp_instances_nodes.get(comp_id).unwrap(),
                        );
                    }
                }

                if let Some(tins) = &instance_k.agreed_upon_unknowns {
                    let tins_node_handle = {
                        let tins_digest = LedgeraDigest::from_serializable(tins).unwrap();
                        let labels = vec![
                            format!("Tins"),
                            format!(
                                "func id : {}..",
                                comp_id
                                    .to_hexadecimal_string()
                                    .chars()
                                    .take(5)
                                    .collect::<String>()
                            ),
                            format!(
                                "tins digest : {}..",
                                tins_digest
                                    .to_hexadecimal_string()
                                    .chars()
                                    .take(5)
                                    .collect::<String>()
                            ),
                            format!(
                                "number of unknowns : {}",
                                tins.v.proposed_unknowns_assignment.len()
                            ),
                        ];
                        let tins_node = make_node(&labels, "#f1f500");
                        let tins_node_handle = vg.add_node(tins_node);
                        vg.add_edge(
                            Arrow::simple(""),
                            tins_node_handle,
                            *comp_instances_nodes.get(comp_id).unwrap(),
                        );
                        tins_node_handle
                    };

                    for (arg_id, arg) in &tins.v.proposed_unknowns_assignment {
                        let labels =
                            vec![format!("used as {}th input", arg_id), format!("from Tins")];
                        let usage_node = make_node(&labels, "#974df7");
                        let usage_node_handle = vg.add_node(usage_node);
                        let tuple = (
                            // comp_id of the one that generated the proof of storage, not necessarily that of the comp that uses it as input
                            arg.pos.v.function_instance_identifier.clone(),
                            arg.pos.v.data_kind.clone(),
                        );
                        if let Some(data_ref_node_handle) = data_reference_nodes.get(&tuple) {
                            vg.add_edge(
                                Arrow::simple(""),
                                *data_ref_node_handle,
                                usage_node_handle,
                            );
                        }
                        vg.add_edge(Arrow::simple(""), usage_node_handle, tins_node_handle);
                    }
                }
            }

            // ***
        }
    }

    if data_nodes.is_empty() && data_reference_nodes.is_empty() && comp_instances_nodes.is_empty() {
        // the graph is empty
    } else {
        let mut svg = SVGWriter::new();
        vg.do_it(false, false, false, &mut svg);

        let _ = save_to_file(filename_to_print_graph, &svg.finalize());
    }
}

fn make_node(labels: &[String], line_color: &str) -> Element {
    Element::create(
        ShapeKind::new_box(&labels.join("\n")),
        StyleAttr::new(
            Color::from_name(line_color).unwrap(),
            2,
            Color::from_name("#FFFFFF"),
            0,
            15,
        ),
        Orientation::LeftToRight,
        Point::new(
            (15 * labels.iter().map(|x| x.len()).max().unwrap()) as f64 * 0.5,
            (15 * labels.len()) as f64,
        ),
    )
}
