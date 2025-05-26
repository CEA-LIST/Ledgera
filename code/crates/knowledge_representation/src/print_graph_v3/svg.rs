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

use std::collections::HashMap;

use super::layout::{ColBox, ComputedLayout, GroupBox, NodeBox, NodeShape, TextAlign, NODE_PAD_X};

const FONT_FAMILY: &str = "monospace";
const FONT_SIZE: f64 = 11.0;
const TITLE_FONT_SIZE: f64 = 13.0;
const NODE_STROKE: &str = "#333333";
const NODE_STROKE_W: f64 = 1.5;
const COL_STROKE: &str = "#555555";
const COL_STROKE_W: f64 = 1.5;
const GROUP_STROKE: &str = "#444444";
const GROUP_STROKE_W: f64 = 2.0;
const RED: &str = "#CC0000";
const BLUE: &str = "#0000CC";
const EDGE_W: f64 = 1.5;

fn defs() -> String {
    format!(
        r#"<defs>
  <marker id="arr-red" markerWidth="8" markerHeight="6" refX="7" refY="3" orient="auto">
    <polygon points="0 0, 8 3, 0 6" fill="{RED}"/>
  </marker>
  <marker id="arr-blue" markerWidth="8" markerHeight="6" refX="7" refY="3" orient="auto">
    <polygon points="0 0, 8 3, 0 6" fill="{BLUE}"/>
  </marker>
</defs>"#,
        RED = RED,
        BLUE = BLUE,
    )
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn draw_col_box(cb: &ColBox) -> String {
    format!(
        "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" rx=\"5\" \
         fill=\"#F8F8F8\" stroke=\"{stroke}\" stroke-width=\"{sw:.1}\"/>\n\
         <text x=\"{tx:.1}\" y=\"{ty:.1}\" font-family=\"{ff}\" font-size=\"{fs:.0}\" \
         text-anchor=\"middle\" font-weight=\"bold\" fill=\"#444\">{label}</text>",
        x = cb.x,
        y = cb.y,
        w = cb.w,
        h = cb.h,
        stroke = COL_STROKE,
        sw = COL_STROKE_W,
        tx = cb.x + cb.w / 2.0,
        ty = cb.y + 18.0,
        ff = FONT_FAMILY,
        fs = TITLE_FONT_SIZE,
        label = escape_xml(&cb.label),
    )
}

fn draw_group_box(gb: &GroupBox) -> String {
    format!(
        "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" rx=\"3\" \
         fill=\"#EBEBEB\" stroke=\"{stroke}\" stroke-width=\"{sw:.1}\" stroke-dasharray=\"5,3\"/>",
        x = gb.x,
        y = gb.y,
        w = gb.w,
        h = gb.h,
        stroke = GROUP_STROKE,
        sw = GROUP_STROKE_W,
    )
}

fn draw_shape(nb: &NodeBox) -> String {
    let x = nb.cx - nb.w / 2.0;
    let y = nb.cy - nb.h / 2.0;
    match nb.shape {
        NodeShape::Rect => format!(
            "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" rx=\"3\" \
             fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{sw:.1}\"/>",
            x = x,
            y = y,
            w = nb.w,
            h = nb.h,
            fill = nb.fill,
            stroke = NODE_STROKE,
            sw = NODE_STROKE_W,
        ),
        NodeShape::Ellipse => format!(
            "<ellipse cx=\"{cx:.1}\" cy=\"{cy:.1}\" rx=\"{rx:.1}\" ry=\"{ry:.1}\" \
             fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{sw:.1}\"/>",
            cx = nb.cx,
            cy = nb.cy,
            rx = nb.w / 2.0,
            ry = nb.h / 2.0,
            fill = nb.fill,
            stroke = NODE_STROKE,
            sw = NODE_STROKE_W,
        ),
        NodeShape::Hexagon => {
            // Flat-top hexagon: six points from center
            let (cx, cy, hw, hh) = (nb.cx, nb.cy, nb.w / 2.0, nb.h / 2.0);
            let pts = format!(
                "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                cx - hw / 2.0,
                cy - hh,
                cx + hw / 2.0,
                cy - hh,
                cx + hw,
                cy,
                cx + hw / 2.0,
                cy + hh,
                cx - hw / 2.0,
                cy + hh,
                cx - hw,
                cy,
            );
            format!(
                "<polygon points=\"{pts}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{sw:.1}\"/>",
                pts = pts, fill = nb.fill, stroke = NODE_STROKE, sw = NODE_STROKE_W,
            )
        }
    }
}

fn draw_label(nb: &NodeBox) -> String {
    if nb.label_lines.is_empty() {
        return String::new();
    }
    let total_text_h = FONT_SIZE * nb.label_lines.len() as f64;
    let first_baseline_y = nb.cy - total_text_h / 2.0 + FONT_SIZE * 0.85;

    let (anchor, text_x) = match nb.text_align {
        TextAlign::Left => ("start", nb.cx - nb.w / 2.0 + NODE_PAD_X),
        TextAlign::Center => ("middle", nb.cx),
        TextAlign::Right => ("end", nb.cx + nb.w / 2.0 - NODE_PAD_X),
    };

    let mut s = format!(
        "<text x=\"{tx:.1}\" y=\"{y:.1}\" font-family=\"{ff}\" font-size=\"{fs:.0}\" \
         text-anchor=\"{anchor}\" fill=\"black\">",
        tx = text_x,
        y = first_baseline_y,
        ff = FONT_FAMILY,
        fs = FONT_SIZE,
        anchor = anchor,
    );
    for (i, line) in nb.label_lines.iter().enumerate() {
        let dy = if i == 0 { 0.0 } else { FONT_SIZE };
        s.push_str(&format!(
            "<tspan x=\"{tx:.1}\" dy=\"{dy:.1}\">{text}</tspan>",
            tx = text_x,
            dy = dy,
            text = escape_xml(line),
        ));
    }
    s.push_str("</text>");
    s
}

fn draw_node(nb: &NodeBox) -> String {
    format!("{}\n{}", draw_shape(nb), draw_label(nb))
}

/// Cross-cluster edge: red bezier, forward (left→right) or backward (right→left).
fn draw_cross_edge(from: &NodeBox, to: &NodeBox, dashed: bool) -> String {
    let forward = from.cx < to.cx;
    let (fx, fy, tx, ty) = if forward {
        (from.cx + from.w / 2.0, from.cy, to.cx - to.w / 2.0, to.cy)
    } else {
        (from.cx - from.w / 2.0, from.cy, to.cx + to.w / 2.0, to.cy)
    };
    let ctrl_bend = (tx - fx).abs().max((ty - fy).abs()) / 2.5;
    let (c1x, c1y, c2x, c2y) = if forward {
        (fx + ctrl_bend, fy, tx - ctrl_bend, ty)
    } else {
        (fx - ctrl_bend, fy, tx + ctrl_bend, ty)
    };
    let dash = if dashed {
        " stroke-dasharray=\"6,3\""
    } else {
        ""
    };
    format!(
        "<path d=\"M {fx:.1},{fy:.1} C {c1x:.1},{c1y:.1} {c2x:.1},{c2y:.1} {tx:.1},{ty:.1}\" \
         fill=\"none\" stroke=\"{RED}\" stroke-width=\"{ew:.1}\"{dash} \
         marker-end=\"url(#arr-red)\"/>",
        fx = fx,
        fy = fy,
        c1x = c1x,
        c1y = c1y,
        c2x = c2x,
        c2y = c2y,
        tx = tx,
        ty = ty,
        RED = RED,
        ew = EDGE_W,
        dash = dash,
    )
}

/// Intra-group edge: blue straight line top-to-bottom between nodes in the same fi_group.
fn draw_intra_edge(from: &NodeBox, to: &NodeBox) -> String {
    let fx = from.cx;
    let fy = from.cy + from.h / 2.0;
    let tx = to.cx;
    let ty = to.cy - to.h / 2.0;
    format!(
        "<line x1=\"{fx:.1}\" y1=\"{fy:.1}\" x2=\"{tx:.1}\" y2=\"{ty:.1}\" \
         stroke=\"{BLUE}\" stroke-width=\"{ew:.1}\" marker-end=\"url(#arr-blue)\"/>",
        fx = fx,
        fy = fy,
        tx = tx,
        ty = ty,
        BLUE = BLUE,
        ew = EDGE_W,
    )
}

pub fn render_svg(layout: &ComputedLayout) -> String {
    let w = layout.total_w.ceil() as i64;
    let h = layout.total_h.ceil() as i64;

    let mut out = String::with_capacity(8192);
    out.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\">\n"
    ));
    out.push_str(&defs());
    out.push('\n');
    out.push_str(&format!(
        "<rect x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\" fill=\"white\"/>\n"
    ));

    for cb in &layout.col_boxes {
        out.push_str(&draw_col_box(cb));
        out.push('\n');
    }
    for gb in &layout.group_boxes {
        out.push_str(&draw_group_box(gb));
        out.push('\n');
    }

    let node_map: HashMap<&str, &NodeBox> =
        layout.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    for (from_id, to_id, dashed) in &layout.cross_edges {
        if let (Some(from), Some(to)) =
            (node_map.get(from_id.as_str()), node_map.get(to_id.as_str()))
        {
            out.push_str(&draw_cross_edge(from, to, *dashed));
            out.push('\n');
        }
    }
    for (from_id, to_id) in &layout.intra_edges {
        if let (Some(from), Some(to)) =
            (node_map.get(from_id.as_str()), node_map.get(to_id.as_str()))
        {
            out.push_str(&draw_intra_edge(from, to));
            out.push('\n');
        }
    }

    for nb in &layout.nodes {
        out.push_str(&draw_node(nb));
        out.push('\n');
    }

    out.push_str("</svg>\n");
    out
}
