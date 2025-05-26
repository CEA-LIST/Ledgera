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

pub const MIN_NODE_W: f64 = 60.0;
pub const NODE_PAD_X: f64 = 10.0;
pub const CHAR_W: f64 = 6.6; // monospace 11px char width estimate
const LINE_H: f64 = 14.0;
const NODE_PAD_Y: f64 = 7.0;
const NODE_GAP: f64 = 10.0;
pub const GROUP_PAD: f64 = 12.0;
const GROUP_GAP: f64 = 18.0;
const COL_TOP_PAD: f64 = 28.0;
const COL_BOT_PAD: f64 = 14.0;
pub const COL_W_PAD: f64 = 14.0;
pub const COL_GAP: f64 = 90.0;
pub const MARGIN: f64 = 30.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NodeShape {
    Rect,
    Ellipse,
    Hexagon,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

pub fn estimate_node_w(label_lines: &[String]) -> f64 {
    let max_chars = label_lines
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0);
    (max_chars as f64 * CHAR_W + 2.0 * NODE_PAD_X).max(MIN_NODE_W)
}

pub struct LayoutNode {
    pub id: String,
    pub label_lines: Vec<String>,
    pub shape: NodeShape,
    pub fill: &'static str,
    pub text_align: TextAlign,
}

pub struct FiGroup {
    pub moniker: String,
    pub nodes: Vec<LayoutNode>,
    /// (from_index, to_index) into `nodes`; rendered as blue solid edges
    pub intra_edges: Vec<(usize, usize)>,
}

pub struct LayoutGraph {
    pub fi_groups: Vec<FiGroup>,
    pub stored_as_nodes: Vec<LayoutNode>,
    pub storage_nodes: Vec<LayoutNode>,
    /// (from_id, to_id, dashed) — always red
    pub edges: Vec<(String, String, bool)>,
}

pub struct NodeBox {
    pub id: String,
    pub cx: f64,
    pub cy: f64,
    pub w: f64,
    pub h: f64,
    pub label_lines: Vec<String>,
    pub shape: NodeShape,
    pub fill: &'static str,
    pub text_align: TextAlign,
}

pub struct GroupBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

pub struct ColBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub label: String,
}

pub struct ComputedLayout {
    pub total_w: f64,
    pub total_h: f64,
    pub col_boxes: Vec<ColBox>,
    pub group_boxes: Vec<GroupBox>,
    pub nodes: Vec<NodeBox>,
    /// (from_id, to_id, dashed) red cross-cluster edges
    pub cross_edges: Vec<(String, String, bool)>,
    /// (from_id, to_id) blue intra-group edges
    pub intra_edges: Vec<(String, String)>,
}

fn node_h(n_lines: usize) -> f64 {
    2.0 * NODE_PAD_Y + LINE_H * (n_lines.max(1) as f64)
}

/// Returns (vec of (cy_relative_to_content_start, height), total_content_height).
fn stack_nodes(nodes: &[LayoutNode]) -> (Vec<(f64, f64)>, f64) {
    let mut positions = Vec::with_capacity(nodes.len());
    let mut cursor = 0.0_f64;
    for (i, node) in nodes.iter().enumerate() {
        if i > 0 {
            cursor += NODE_GAP;
        }
        let h = node_h(node.label_lines.len());
        positions.push((cursor + h / 2.0, h));
        cursor += h;
    }
    (positions, cursor)
}

pub fn compute_layout(g: &LayoutGraph) -> ComputedLayout {
    // Compute column node widths (uniform per column = max across all nodes in that column)
    let log_node_w = g
        .fi_groups
        .iter()
        .flat_map(|grp| grp.nodes.iter())
        .map(|n| estimate_node_w(&n.label_lines))
        .fold(MIN_NODE_W, f64::max);
    let sa_node_w = g
        .stored_as_nodes
        .iter()
        .map(|n| estimate_node_w(&n.label_lines))
        .fold(MIN_NODE_W, f64::max);
    let st_node_w = g
        .storage_nodes
        .iter()
        .map(|n| estimate_node_w(&n.label_lines))
        .fold(MIN_NODE_W, f64::max);

    // Per fi_group: (group_height, vec of (local_cy_within_group, node_h))
    struct PerFiGroup {
        pub group_height: f64,
        pub nodes_positions: Vec<(f64, f64)>,
    }

    let mut fi_geom: Vec<PerFiGroup> = Vec::new();
    let mut log_content_h = 0.0_f64;

    for (gi, grp) in g.fi_groups.iter().enumerate() {
        if gi > 0 {
            log_content_h += GROUP_GAP;
        }
        let mut nodes_positions: Vec<(f64, f64)> = Vec::new();
        let mut cursor = GROUP_PAD;
        for (ni, node) in grp.nodes.iter().enumerate() {
            if ni > 0 {
                cursor += NODE_GAP;
            }
            let h = node_h(node.label_lines.len());
            nodes_positions.push((cursor + h / 2.0, h));
            cursor += h;
        }
        let group_height = if grp.nodes.is_empty() {
            2.0 * GROUP_PAD
        } else {
            cursor + GROUP_PAD
        };
        log_content_h += group_height;
        fi_geom.push(PerFiGroup {
            group_height,
            nodes_positions,
        });
    }

    let log_col_w = log_node_w + 2.0 * COL_W_PAD + 2.0 * GROUP_PAD;
    let log_col_h = COL_TOP_PAD + log_content_h + COL_BOT_PAD;

    let (sa_positions, sa_content_h) = stack_nodes(&g.stored_as_nodes);
    let sa_col_w = sa_node_w + 2.0 * COL_W_PAD;
    let sa_col_h = COL_TOP_PAD + sa_content_h + COL_BOT_PAD;

    let (st_positions, st_content_h) = stack_nodes(&g.storage_nodes);
    let st_col_w = st_node_w + 2.0 * COL_W_PAD;
    let st_col_h = COL_TOP_PAD + st_content_h + COL_BOT_PAD;

    let log_col_x = MARGIN;
    let sa_col_x = log_col_x + log_col_w + COL_GAP;
    let st_col_x = sa_col_x + sa_col_w + COL_GAP;
    let col_top_y = MARGIN;

    let max_col_h = log_col_h.max(sa_col_h).max(st_col_h);
    let total_h = col_top_y + max_col_h + MARGIN;
    let total_w = st_col_x + st_col_w + MARGIN;

    let col_boxes = vec![
        ColBox {
            x: log_col_x,
            y: col_top_y,
            w: log_col_w,
            h: log_col_h,
            label: "log".to_string(),
        },
        ColBox {
            x: sa_col_x,
            y: col_top_y,
            w: sa_col_w,
            h: sa_col_h,
            label: "stored as".to_string(),
        },
        ColBox {
            x: st_col_x,
            y: col_top_y,
            w: st_col_w,
            h: st_col_h,
            label: "storage".to_string(),
        },
    ];

    let mut nodes: Vec<NodeBox> = Vec::new();
    let mut group_boxes: Vec<GroupBox> = Vec::new();
    let mut intra_edges: Vec<(String, String)> = Vec::new();

    // Log column
    let group_box_x = log_col_x + COL_W_PAD;
    let group_box_w = log_node_w + 2.0 * GROUP_PAD;
    let log_node_cx = group_box_x + GROUP_PAD + log_node_w / 2.0;
    let mut group_y = col_top_y + COL_TOP_PAD;

    for (gi, grp) in g.fi_groups.iter().enumerate() {
        if gi > 0 {
            group_y += GROUP_GAP;
        }
        let PerFiGroup {
            group_height,
            nodes_positions,
        } = &fi_geom[gi];
        group_boxes.push(GroupBox {
            x: group_box_x,
            y: group_y,
            w: group_box_w,
            h: *group_height,
        });
        for (ni, node) in grp.nodes.iter().enumerate() {
            let (local_cy, h) = nodes_positions[ni];
            nodes.push(NodeBox {
                id: node.id.clone(),
                cx: log_node_cx,
                cy: group_y + local_cy,
                w: log_node_w,
                h,
                label_lines: node.label_lines.clone(),
                shape: node.shape,
                fill: node.fill,
                text_align: node.text_align,
            });
        }
        for &(from_idx, to_idx) in &grp.intra_edges {
            intra_edges.push((grp.nodes[from_idx].id.clone(), grp.nodes[to_idx].id.clone()));
        }
        group_y += group_height;
    }

    // Stored-as column
    let sa_node_cx = sa_col_x + COL_W_PAD + sa_node_w / 2.0;
    for (i, node) in g.stored_as_nodes.iter().enumerate() {
        let (local_cy, h) = sa_positions[i];
        nodes.push(NodeBox {
            id: node.id.clone(),
            cx: sa_node_cx,
            cy: col_top_y + COL_TOP_PAD + local_cy,
            w: sa_node_w,
            h,
            label_lines: node.label_lines.clone(),
            shape: node.shape,
            fill: node.fill,
            text_align: node.text_align,
        });
    }

    // Storage column
    let st_node_cx = st_col_x + COL_W_PAD + st_node_w / 2.0;
    for (i, node) in g.storage_nodes.iter().enumerate() {
        let (local_cy, h) = st_positions[i];
        nodes.push(NodeBox {
            id: node.id.clone(),
            cx: st_node_cx,
            cy: col_top_y + COL_TOP_PAD + local_cy,
            w: st_node_w,
            h,
            label_lines: node.label_lines.clone(),
            shape: node.shape,
            fill: node.fill,
            text_align: node.text_align,
        });
    }

    ComputedLayout {
        total_w,
        total_h,
        col_boxes,
        group_boxes,
        nodes,
        cross_edges: g.edges.clone(),
        intra_edges,
    }
}
