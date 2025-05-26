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

use std::collections::{BTreeSet, HashMap};

use ledgera_pki::manager::SerdeSerializable64BitsSignature;
use ledgera_types::{
    app_template::{
        input::LedgeraInputArgument, operation::LedgeraAtomicOperation,
        template::LedgeraApplicationTemplate,
    },
    digest::LedgeraDigest,
    votes::{vout::LedgeraFunctionInstanceOutputKind, vsto::PersistentDataKind},
};

use crate::{know::LedgeraKnowledgeRepresentation, printer::LedgeraComputationItemsPrinter};

use super::layout::{compute_layout, FiGroup, LayoutGraph, LayoutNode, NodeShape, TextAlign};
use super::svg::render_svg;

fn decl_node_name(c_moniker: &str) -> String {
    format!("decl_{}", c_moniker)
}

fn agr_node_name(c_moniker: &str) -> String {
    format!("agr_{}", c_moniker)
}

fn integ_node_name(c_moniker: &str) -> String {
    format!("integ_{}", c_moniker)
}

fn ref_node_name(c_moniker: &str, write_kind: &PersistentDataKind) -> String {
    match write_kind {
        PersistentDataKind::Input(x) => format!("ref_{}_i{}", c_moniker, x),
        PersistentDataKind::Output => format!("ref_{}_o", c_moniker),
    }
}

fn data_node_name(d_moniker: &str) -> String {
    format!("data_{}", d_moniker)
}

fn build_layout_graph<LAT, Printer>(
    k: &LedgeraKnowledgeRepresentation<LAT>,
    c_monikers: &HashMap<String, SerdeSerializable64BitsSignature>,
    d_monikers: &HashMap<String, LedgeraDigest>,
) -> LayoutGraph
where
    LAT: LedgeraApplicationTemplate,
    Printer: LedgeraComputationItemsPrinter<LAT>,
{
    let fid_to_c_moniker: HashMap<&SerdeSerializable64BitsSignature, &str> = c_monikers
        .iter()
        .map(|(m, fid)| (fid, m.as_str()))
        .collect();

    // Collect (c_moniker, write_kind, d_moniker) triples
    let mut writes: BTreeSet<(String, PersistentDataKind, String)> = BTreeSet::new();
    for (d_moniker, data_digest) in d_monikers {
        if let Some(data_k) = k.per_data_value.get(data_digest) {
            for ((fid, kind), poss) in &data_k.proofs_of_storage {
                if poss.iter().next().is_some() {
                    if let Some(c_mon) = fid_to_c_moniker.get(fid) {
                        writes.insert((c_mon.to_string(), kind.clone(), d_moniker.clone()));
                    }
                }
            }
        }
    }

    let mut has_agreement: HashMap<String, bool> = HashMap::new();
    let mut has_integrity: HashMap<String, Option<String>> = HashMap::new();
    for (c_moniker, fid) in c_monikers {
        if let Some(fid_k) = k.per_function_instance.get(fid) {
            has_agreement.insert(c_moniker.clone(), fid_k.agreed_upon_unknowns.is_some());
            let got_integ =
                fid_k
                    .proofs_of_result_integrity
                    .iter()
                    .next()
                    .map(|x| match &x.v.result_kind {
                        LedgeraFunctionInstanceOutputKind::ComputedOutput {
                            is_output_persistent: _,
                            output_digest,
                        } => {
                            let digest_str = output_digest
                                .to_hexadecimal_string()
                                .chars()
                                .take(8)
                                .collect::<String>();
                            format!("output digest : {}..", digest_str)
                        }
                        LedgeraFunctionInstanceOutputKind::TaggedInputs => {
                            "multi-party Tag integrity checked".to_string()
                        }
                    });
            has_integrity.insert(c_moniker.clone(), got_integ);
        } else {
            has_agreement.insert(c_moniker.clone(), false);
            has_integrity.insert(c_moniker.clone(), None);
        }
    }

    // Log column: fi_groups ordered by function_instances_order_in_log
    let ordered_monikers: Vec<&str> = k
        .function_instances_order_in_log
        .iter()
        .filter_map(|fid| fid_to_c_moniker.get(fid).copied())
        .collect();

    let mut fi_groups: Vec<FiGroup> = Vec::new();
    for &c_moniker in &ordered_monikers {
        let fid = match c_monikers.get(c_moniker) {
            Some(f) => f,
            None => continue,
        };
        let fid_k = k.per_function_instance.get(fid);

        // --- Tfun label ---
        let mut decl_label = vec![format!("Tfun : declaration of {}", c_moniker)];
        if let Some(fid_k) = fid_k {
            if let Some(spec) = &fid_k.spec {
                match &spec.operation {
                    LedgeraAtomicOperation::TagInputs(tag) => {
                        decl_label.push(format!("tag : {}", Printer::print_tag(tag)));
                    }
                    LedgeraAtomicOperation::ComputeOutput {
                        comp,
                        is_output_persistent,
                    } => {
                        decl_label.push(format!("compute : {}", Printer::print_computation(comp)));
                        decl_label.push(format!("persistent output : {:?}", is_output_persistent));
                    }
                }
                // Known inputs: one line per argument position
                for (i, arg) in spec.arguments.iter().enumerate() {
                    match arg {
                        LedgeraInputArgument::RawValue {
                            is_input_persistent,
                            value,
                        } => {
                            decl_label.push(format!(
                                "in[{}].value : {}",
                                i,
                                Printer::print_value(value)
                            ));
                            decl_label.push(format!(
                                "in[{}].persistence : {}",
                                i,
                                if *is_input_persistent {
                                    "true"
                                } else {
                                    "false"
                                }
                            ));
                        }
                        LedgeraInputArgument::ReferenceToStorage(pos) => {
                            let src = fid_to_c_moniker
                                .get(&pos.v.function_instance_identifier)
                                .copied()
                                .unwrap_or("?");
                            decl_label.push(format!(
                                "in[{}].reference : {} of {}",
                                i, pos.v.data_kind, src,
                            ));
                        }
                        LedgeraInputArgument::Unknown(pred) => {
                            decl_label.push(format!(
                                "in[{}].predicate : {}",
                                i,
                                Printer::print_local_predicate(pred),
                            ));
                        }
                    }
                }
            }
        }

        let mut nodes: Vec<LayoutNode> = vec![LayoutNode {
            id: decl_node_name(c_moniker),
            label_lines: decl_label,
            shape: NodeShape::Rect,
            fill: "#76EE00",
            text_align: TextAlign::Left,
        }];
        let mut intra_edges: Vec<(usize, usize)> = Vec::new();

        let has_agr = *has_agreement.get(c_moniker).unwrap_or(&false);
        let has_integ = has_integrity.get(c_moniker).unwrap_or(&None);

        if has_agr {
            // --- Tins label ---
            let mut agr_label = vec![format!("Tins : agreement of {}", c_moniker)];
            if let Some(fid_k) = fid_k {
                if let Some(tins) = &fid_k.agreed_upon_unknowns {
                    // Decided unknowns: one line per resolved argument position
                    for (arg_idx, assignment) in &tins.v.proposed_unknowns_assignment {
                        let src = fid_to_c_moniker
                            .get(&assignment.pos.v.function_instance_identifier)
                            .copied()
                            .unwrap_or("?");
                        agr_label.push(format!(
                            "in[{}].reference : {} of {}",
                            arg_idx, assignment.pos.v.data_kind, src,
                        ));
                    }
                }
            }
            intra_edges.push((nodes.len() - 1, nodes.len()));
            nodes.push(LayoutNode {
                id: agr_node_name(c_moniker),
                label_lines: agr_label,
                shape: NodeShape::Rect,
                fill: "#DA70D6",
                text_align: TextAlign::Left,
            });
        }
        if let Some(integ_inner_lab) = has_integ {
            let integ_label = vec![
                format!("Tout : integrity of {}", c_moniker),
                integ_inner_lab.clone(),
            ];
            intra_edges.push((nodes.len() - 1, nodes.len()));
            nodes.push(LayoutNode {
                id: integ_node_name(c_moniker),
                label_lines: integ_label,
                shape: NodeShape::Rect,
                fill: "#FFFF00",
                text_align: TextAlign::Left,
            });
        }

        fi_groups.push(FiGroup {
            moniker: c_moniker.to_string(),
            nodes,
            intra_edges,
        });
    }

    // Stored-as column
    let stored_as_nodes: Vec<LayoutNode> = writes
        .iter()
        .map(|(c_moniker, write_kind, _)| {
            let label_lines = match write_kind {
                PersistentDataKind::Input(x) => {
                    vec![format!("{}th input", x), format!("of {}", c_moniker)]
                }
                PersistentDataKind::Output => {
                    vec!["output".to_string(), format!("of {}", c_moniker)]
                }
            };
            LayoutNode {
                id: ref_node_name(c_moniker, write_kind),
                label_lines,
                shape: NodeShape::Ellipse,
                fill: "#FFA500",
                text_align: TextAlign::Center,
            }
        })
        .collect();

    // Storage column — sorted by moniker for deterministic order
    let mut sorted_d_monikers: Vec<(&String, &LedgeraDigest)> = d_monikers.iter().collect();
    sorted_d_monikers.sort_by_key(|(m, _)| m.as_str());
    let storage_nodes: Vec<LayoutNode> = sorted_d_monikers
        .iter()
        .map(|(d_moniker, data_digest)| {
            let mut label_lines = vec![
                d_moniker.to_string(),
                format!(
                    "digest: {}..",
                    data_digest
                        .to_hexadecimal_string()
                        .chars()
                        .take(8)
                        .collect::<String>()
                ),
            ];
            if let Some(data_k) = k.per_data_value.get(data_digest) {
                if let Some(value) = &data_k.data_value {
                    label_lines.push(format!("value: {}", Printer::print_value(value)));
                }
            }
            LayoutNode {
                id: data_node_name(d_moniker),
                label_lines,
                shape: NodeShape::Hexagon,
                fill: "#00FFFF",
                text_align: TextAlign::Center,
            }
        })
        .collect();

    // Cross-cluster edges
    let mut edges: Vec<(String, String, bool)> = Vec::new();

    // Log → Stored-as: solid red
    for (c_moniker, write_kind, _) in &writes {
        let source = match write_kind {
            PersistentDataKind::Input(_) => decl_node_name(c_moniker),
            PersistentDataKind::Output => {
                if has_integrity.get(c_moniker).unwrap_or(&None).is_some() {
                    integ_node_name(c_moniker)
                } else {
                    decl_node_name(c_moniker)
                }
            }
        };
        edges.push((source, ref_node_name(c_moniker, write_kind), false));
    }

    // Stored-as → Storage: solid red
    for (c_moniker, write_kind, d_moniker) in &writes {
        edges.push((
            ref_node_name(c_moniker, write_kind),
            data_node_name(d_moniker),
            false,
        ));
    }

    // Stored-as → Log: dashed red (consumption)
    for (c_moniker, fid) in c_monikers {
        if let Some(fid_k) = k.per_function_instance.get(fid) {
            if let Some(spec) = &fid_k.spec {
                for arg in &spec.arguments {
                    if let LedgeraInputArgument::ReferenceToStorage(pos) = arg {
                        if let Some(pos_c_moniker) =
                            fid_to_c_moniker.get(&pos.v.function_instance_identifier)
                        {
                            edges.push((
                                ref_node_name(pos_c_moniker, &pos.v.data_kind),
                                decl_node_name(c_moniker),
                                true,
                            ));
                        }
                    }
                }
            }
            if let Some(tins) = &fid_k.agreed_upon_unknowns {
                let target = if *has_agreement.get(c_moniker).unwrap_or(&false) {
                    agr_node_name(c_moniker)
                } else if has_integrity.get(c_moniker).unwrap_or(&None).is_some() {
                    integ_node_name(c_moniker)
                } else {
                    decl_node_name(c_moniker)
                };
                for assignment in tins.v.proposed_unknowns_assignment.values() {
                    if let Some(pos_c_moniker) =
                        fid_to_c_moniker.get(&assignment.pos.v.function_instance_identifier)
                    {
                        edges.push((
                            ref_node_name(pos_c_moniker, &assignment.pos.v.data_kind),
                            target.clone(),
                            true,
                        ));
                    }
                }
            }
        }
    }

    LayoutGraph {
        fi_groups,
        stored_as_nodes,
        storage_nodes,
        edges,
    }
}

pub fn print_current_knowledge_as_graph_v3<
    LAT: LedgeraApplicationTemplate,
    Printer: LedgeraComputationItemsPrinter<LAT>,
>(
    filename_to_print_graph: &str,
    k: &LedgeraKnowledgeRepresentation<LAT>,
    c_monikers: &HashMap<String, SerdeSerializable64BitsSignature>,
    d_monikers: &HashMap<String, LedgeraDigest>,
) {
    let graph = build_layout_graph::<LAT, Printer>(k, c_monikers, d_monikers);
    let layout = compute_layout(&graph);
    let svg = render_svg(&layout);
    std::fs::write(filename_to_print_graph, svg).expect("failed to write svg");
}
