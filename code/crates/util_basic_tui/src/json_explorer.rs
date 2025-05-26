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

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use serde_json::Value;

fn value_color(v: &Value) -> Color {
    match v {
        Value::String(_) => Color::Green,
        Value::Number(_) => Color::Cyan,
        Value::Bool(_) => Color::Yellow,
        Value::Null => Color::DarkGray,
        Value::Object(_) | Value::Array(_) => Color::LightBlue,
    }
}

// Returns a semantic color for a value based on what the key represents.
// Falls back to value_color when no semantic match is found.
fn semantic_color(key: &str, val: &Value) -> Color {
    let sem = match key.find(':') {
        Some(i) => &key[i + 1..],
        None => key,
    };
    match sem {
        // ── top-level categories ──────────────────────────────────────────────
        "data" => Color::LightGreen,
        "function instances" => Color::LightMagenta,

        // ── identity / digest ─────────────────────────────────────────────────
        "digest"
        | "value_digest"
        | "function_instance_id"
        | "function_instance"
        | "pouav_digest" => Color::Cyan,

        // ── proof kinds ───────────────────────────────────────────────────────
        "proof_kind" => match val.as_str().unwrap_or("") {
            "Proof Of Storage" => Color::LightGreen,
            "Proof Of Declaration" => Color::LightBlue,
            "Proof Of Assignment" => Color::LightYellow,
            "Proof Of Integrity" => Color::LightMagenta,
            _ => Color::White,
        },

        // ── input / output kinds ──────────────────────────────────────────────
        "input_kind" => match val.as_str().unwrap_or("") {
            "RawValue" => Color::Yellow,
            "ReferenceToStorage" => Color::Cyan,
            "Unknown" => Color::DarkGray,
            _ => Color::White,
        },
        "is_input_persistent" | "is_output_persistent" => match val.as_str().unwrap_or("") {
            "true" => Color::LightGreen,
            _ => Color::DarkGray,
        },
        "output_digest" => Color::Cyan,
        "output_value" | "value" => Color::LightGreen,
        "tag" => Color::Magenta,

        // ── computation / operation ───────────────────────────────────────────
        "identity function tag" | "compute function" => Color::Magenta,
        "global_predicate" => Color::LightYellow,
        "result_kind" | "arguments_specification" => Color::LightBlue,

        // ── quorum / signatures ───────────────────────────────────────────────
        "quorum" | "signatures" | "signed_data" => Color::LightRed,

        // ── storage proofs ────────────────────────────────────────────────────
        "Proofs Of Shipment To Storage" | "Proofs Of Declaration" | "Proofs Of Integrity" => {
            Color::LightBlue
        }
        "Proof Of Assignment" => Color::LightYellow,
        "stored_as" | "proposed_by" | "at_indices" => Color::DarkGray,
        "known_arguments" | "unknown_arguments_indices" | "persistent_inputs_indices" => {
            Color::DarkGray
        }

        _ => value_color(val),
    }
}

fn key_label_color(key: &str) -> Color {
    let sem = match key.find(':') {
        Some(i) => &key[i + 1..],
        None => key,
    };
    match sem {
        "data" | "function instances" => Color::White,
        "proof_kind" | "input_kind" => Color::LightCyan,
        "digest" | "value_digest" | "function_instance_id" | "output_digest" | "pouav_digest" => {
            Color::Cyan
        }
        "quorum" | "signatures" | "signed_data" => Color::LightRed,
        "Proofs Of Shipment To Storage"
        | "Proofs Of Declaration"
        | "Proofs Of Integrity"
        | "Proof Of Assignment" => Color::LightMagenta,
        "identity function tag" | "compute function" | "global_predicate" | "tag" => Color::Magenta,
        "is_input_persistent" | "is_output_persistent" => Color::Yellow,
        _ => Color::White,
    }
}

#[derive(Debug, Clone)]
pub enum PathItem {
    Key(String),
    Index(usize),
}

#[derive(Debug, Default)]
pub struct ExplorerState {
    pub path: Vec<PathItem>,
    pub selections: Vec<usize>, // per depth
}

impl ExplorerState {
    pub fn new() -> Self {
        Self {
            path: vec![],
            selections: vec![0], // root level
        }
    }

    pub fn selected(&self) -> usize {
        *self.selections.last().unwrap_or(&0)
    }

    pub fn selected_mut(&mut self) -> &mut usize {
        self.selections.last_mut().unwrap()
    }
}

pub struct JsonExplorer {
    pub state: ExplorerState,
}

impl Default for JsonExplorer {
    fn default() -> Self {
        Self {
            state: ExplorerState::new(),
        }
    }
}

impl JsonExplorer {
    pub fn new() -> Self {
        Self {
            state: ExplorerState::new(),
        }
    }

    // ---------- navigation ----------

    pub fn on_up(&mut self, json: &Value) {
        let current_node = self.current_node(json);
        let items = self.children(current_node);
        if items.is_empty() {
            return;
        }
        let len = items.len();
        let sel = self.state.selected_mut();
        *sel = (*sel + len - 1) % len;
    }

    pub fn on_down(&mut self, json: &Value) {
        let current_node = self.current_node(json);
        let items = self.children(current_node);
        if items.is_empty() {
            return;
        }
        let len = items.len();
        let sel = self.state.selected_mut();
        *sel = (*sel + 1) % len;
    }

    pub fn on_right(&mut self, json: &Value) {
        let node = self.current_node(json);
        let items = self.children(node);

        if items.is_empty() {
            return;
        }

        let sel = self.state.selected();

        match node {
            Value::Object(_) => {
                if items.len() > sel {
                    self.state.path.push(PathItem::Key(items[sel].clone()));
                }
            }
            Value::Array(_) => {
                if items.len() > sel {
                    self.state.path.push(PathItem::Index(sel));
                }
            }
            _ => return, // primitives: do nothing
        }

        self.state.selections.push(0);
    }

    pub fn on_left(&mut self) {
        if !self.state.path.is_empty() {
            self.state.path.pop();
            self.state.selections.pop();
        }
    }

    // ---------- data helpers ----------

    /**
     * Get the current JSON node given the JSON root and the current path.  
     *
     * In case the JSON structure is changed at runtime
     * and the current position does not exist anymore
     * we are moved to the last parent that still exists
     **/
    fn current_node<'a>(&mut self, root: &'a Value) -> &'a Value {
        let mut node = root;

        let mut prune_path = None;
        'iter_path: for (path_index, path_item) in self.state.path.iter().enumerate() {
            match (path_item, node) {
                (PathItem::Key(k), Value::Object(map)) => {
                    if let Some(got_node) = map.get(k) {
                        node = got_node
                    } else {
                        prune_path = Some(path_index);
                        break 'iter_path;
                    }
                }
                (PathItem::Index(i), Value::Array(arr)) => {
                    if arr.len() > *i {
                        node = &arr[*i];
                    } else {
                        prune_path = Some(path_index);
                        break 'iter_path;
                    }
                }
                _ => {
                    break 'iter_path;
                }
            }
        }
        if let Some(prune_at) = prune_path {
            self.state.path.truncate(prune_at);
            self.state.selections.truncate(prune_at);
            self.state.selections.push(0);
        }

        node
    }

    fn children(&self, node: &Value) -> Vec<String> {
        match node {
            Value::Object(map) => map.keys().cloned().collect(),
            Value::Array(arr) => (0..arr.len()).map(|i| i.to_string()).collect(),
            _ => vec![],
        }
    }

    fn breadcrumb(&self) -> String {
        if self.state.path.is_empty() {
            return "root".into();
        }

        let parts: Vec<String> = self
            .state
            .path
            .iter()
            .map(|p| match p {
                PathItem::Key(k) => k.clone(),
                PathItem::Index(i) => format!("[{}]", i),
            })
            .collect();

        format!("root > {}", parts.join(" > "))
    }

    fn format_item(&self, node: &Value, key: &str) -> Line<'static> {
        match node {
            Value::Object(map) => {
                let val = &map[key];
                let key_span =
                    Span::styled(key.to_string(), Style::default().fg(key_label_color(key)));
                match val {
                    Value::Object(_) => Line::from(vec![
                        key_span,
                        Span::styled(" >", Style::default().fg(semantic_color(key, val))),
                    ]),
                    Value::Array(_) => Line::from(vec![
                        key_span,
                        Span::styled(
                            format!(" [{}]", val.as_array().unwrap().len()),
                            Style::default().fg(semantic_color(key, val)),
                        ),
                    ]),
                    _ => Line::from(vec![
                        Span::styled(
                            format!("{}: ", key),
                            Style::default().fg(key_label_color(key)),
                        ),
                        Span::styled(preview(val), Style::default().fg(semantic_color(key, val))),
                    ]),
                }
            }
            Value::Array(arr) => {
                let idx: usize = key.parse().unwrap();
                let val = &arr[idx];
                match val {
                    Value::Object(_) => Line::from(vec![
                        Span::raw(format!("[{}]", idx)),
                        Span::styled(" >", Style::default().fg(Color::LightBlue)),
                    ]),
                    Value::Array(_) => Line::from(vec![
                        Span::raw(format!("[{}]", idx)),
                        Span::styled(
                            format!(" [{}]", val.as_array().unwrap().len()),
                            Style::default().fg(Color::LightBlue),
                        ),
                    ]),
                    _ => Line::from(vec![
                        Span::raw(format!("[{}]: ", idx)),
                        Span::styled(preview(val), Style::default().fg(value_color(val))),
                    ]),
                }
            }
            _ => Line::from(""),
        }
    }

    // ---------- rendering ----------

    pub fn render(&mut self, f: &mut Frame, area: Rect, json: &Value) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        let breadcrumb =
            Paragraph::new(self.breadcrumb()).style(Style::default().fg(Color::Yellow));
        f.render_widget(breadcrumb, chunks[0]);

        // list
        let node = self.current_node(json);
        let items = self.children(node);

        let list_items: Vec<ListItem> = items
            .iter()
            .map(|k| ListItem::new(self.format_item(node, k)))
            .collect::<Vec<_>>();

        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(self.state.selected()));
        }

        let list = List::new(list_items)
            .block(
                Block::default()
                    .title(Span::styled(
                        "JSON",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL),
            )
            .highlight_symbol(">> ")
            .highlight_style(Style::default().bg(Color::LightBlue).fg(Color::White));

        f.render_stateful_widget(list, chunks[1], &mut state);
    }
}

// ---------- helpers ----------

fn preview(v: &Value) -> String {
    match v {
        Value::String(s) => format!("\"{}\"", s),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".into(),
        _ => "...".into(),
    }
}
