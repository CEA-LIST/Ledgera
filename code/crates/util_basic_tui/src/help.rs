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

use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use ratatui::Frame;

// ── colour palette ────────────────────────────────────────────────────────────
const C_HEADING: Color = Color::Cyan;
const C_KEY: Color = Color::Yellow;
const C_CMD: Color = Color::Green;
const C_ARG: Color = Color::Magenta;
const C_EXAMPLE: Color = Color::LightGreen;
const C_DIM: Color = Color::DarkGray;

fn heading(text: &'static str) -> Line<'static> {
    Line::from(Span::styled(
        text,
        Style::default().fg(C_HEADING).add_modifier(Modifier::BOLD),
    ))
}
fn blank() -> Line<'static> {
    Line::from("")
}
fn plain(text: &'static str) -> Line<'static> {
    Line::from(text)
}
fn key_line(key: &'static str, desc: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::styled(key, Style::default().fg(C_KEY).add_modifier(Modifier::BOLD)),
        Span::raw(desc),
    ])
}
fn cmd_line(cmd: &'static str, desc: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(cmd, Style::default().fg(C_CMD).add_modifier(Modifier::BOLD)),
        Span::raw(desc),
    ])
}
fn arg_line(arg: &'static str, desc: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::raw("      + "),
        Span::styled(arg, Style::default().fg(C_ARG)),
        Span::raw(desc),
    ])
}
fn example_line(ex: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::raw("        "),
        Span::styled(ex, Style::default().fg(C_EXAMPLE)),
    ])
}
fn indent(text: &'static str) -> Line<'static> {
    Line::from(vec![Span::raw("    "), Span::raw(text)])
}
fn dim(text: &'static str) -> Line<'static> {
    Line::from(Span::styled(text, Style::default().fg(C_DIM)))
}

fn tui_help_text() -> Text<'static> {
    Text::from(vec![
        // ── Modes ─────────────────────────────────────────────────────────────
        heading("── MODES ─────────────────────────────────────────────────────"),
        blank(),
        plain("  3 modes: MAIN / EDITION / BROWSING"),
        blank(),
        key_line("  MAIN -> EDITION  ", ": press 'e'"),
        key_line("  MAIN -> BROWSING ", ": press 'b'"),
        key_line("  EDITION -> MAIN  ", ": type 'exit' and ENTER"),
        key_line("  BROWSING -> MAIN ", ": press 'q'"),
        blank(),
        key_line("  'h'", " toggle this help  |  "),
        key_line("  'o'", " toggle service documentation"),
        blank(),
        dim("  In BROWSING mode use ← → ↑ ↓ to navigate the JSON explorer."),
        blank(),
        // ── Navigation & misc ─────────────────────────────────────────────────
        heading("── NAVIGATION & MISC ─────────────────────────────────────────"),
        blank(),
        cmd_line("exit", "          exit EDITION mode"),
        cmd_line("print_graph", "     print graph representation of current knowldege"),
        cmd_line("rename <n1> <n2>", " rename local moniker <n1> to <n2>"),
        blank(),
        // ── Storage & audit ───────────────────────────────────────────────────
        heading("── STORAGE & AUDIT ───────────────────────────────────────────"),
        blank(),
        cmd_line("get_value <name>", "  retrieve the value whose digest is referred to by moniker <name>"),
        cmd_line("audit_value <ref>", " audit a value; <ref> is one of:"),
        arg_line("<value>",  "     raw concrete value"),
        arg_line("^<value>", "    persistent raw value"),
        arg_line("@<name>",  "     storage reference (digest referred to by moniker <name>)"),
        arg_line("*<name>",  "     raw value stored locally under <name>"),
        cmd_line("audit_comp <name>", " audit the function referred to by moniker <name>"),
        blank(),
        // ── Compute ───────────────────────────────────────────────────────────
        heading("── COMPUTE ───────────────────────────────────────────────────"),
        blank(),
        cmd_line("/<comp_op> <args> [-s] [<pred>] [-n <name>]", ""),
        indent("Submit a compute operation."),
        indent("-s  makes the output persistent."),
        indent("-n <name>  assigns local moniker to the executed function instance."),
        indent("[<pred>]  optional global (cross-argument) predicate."),
        indent("<args> is a space-separated list of:"),
        arg_line("<value>",  "     raw concrete value"),
        arg_line("^<value>", "    persistent raw value"),
        arg_line("@<name>",  "     storage reference (digest referred to by <name>)"),
        arg_line("*<name>",  "     raw value stored locally under <name>"),
        arg_line("(<pred>)", "    local predicate (application-specific syntax)"),
        blank(),
        dim("  EXAMPLES with a string concatenation compute operation of arity 2:"),
        example_line("/concat -s apple pie             concatenate the raw strings 'apple' and 'pie' and stores the result"),
        example_line("/concat ^pine @d1                store raw string 'pine' and concatenate it to previously stored value of moniker 'd1' (output not stored)"),
        example_line("/concat ^pine *d1                same but value '@d1' not referenced but directly shipped as a raw string"),
        example_line("/concat foo (>3)                 concatenate 'foo' with an unknown value that has more than 3 chars (multi-party case)"),
        example_line("/concat @a @b [distinct]         concat only if values referred to via 'a' and 'b' are distinct"),
        blank(),
        // ── Push arg ──────────────────────────────────────────────────────────
        heading("── PUSH ARG ──────────────────────────────────────────────────"),
        blank(),
        cmd_line("push_arg <name> {<i>,...} @<data>", ""),
        indent("Propose @<data> as argument at candidate indices {<i>,...}"),
        indent("for the multi-party function referred to by <name>."),
        blank(),
        dim("  EXAMPLES:"),
        example_line("push_arg c {0,1} @v    previously stored value of moniker 'v' proposed as inputs of index 0 or 1 for function of moniker 'c'"),
        blank(),
        // ── Tag ───────────────────────────────────────────────────────────────
        heading("── TAG ───────────────────────────────────────────────────────"),
        blank(),
        cmd_line("|<tag_op> <args> [<pred>] [-n <name>]", ""),
        indent("Submit a tag operation (anchors its inputs without producing an output)."),
        indent("-n <name>  assigns local moniker to the executed function instance."),
        indent("[<pred>]   optional global (cross-argument) predicate."),
        indent("<args> is a space-separated list of:"),
        arg_line("<value>",  "     raw concrete value"),
        arg_line("^<value>", "    persistent raw value"),
        arg_line("@<name>",  "     storage reference (digest referred to by <name>)"),
        arg_line("*<name>",  "     raw value stored locally under <name>"),
        arg_line("(<pred>)", "    local predicate (application-specific syntax)"),
        blank(),
        dim("  EXAMPLES with a tag operation of arity 1:"),
        example_line("|tag hello               tags the raw value 'hello' without storing it"),
        example_line("|tag ^hello              tags the raw value 'hello' and stores it (persistent input)"),
        example_line("|tag @d1                 tags the value referred to by moniker 'd1'"),
        blank(),
    ])
}

pub enum HelpKind {
    None,
    TUI { scroll: u16, visible: u16 },
    Service { scroll: u16, visible: u16 },
}

impl HelpKind {
    pub fn is_open(&self) -> bool {
        !matches!(self, HelpKind::None)
    }

    pub fn h_pressed(&mut self) {
        *self = match self {
            HelpKind::None => HelpKind::TUI {
                scroll: 0,
                visible: 0,
            },
            HelpKind::TUI { .. } => HelpKind::None,
            HelpKind::Service { .. } => HelpKind::TUI {
                scroll: 0,
                visible: 0,
            },
        };
    }

    pub fn o_pressed(&mut self) {
        *self = match self {
            HelpKind::None => HelpKind::Service {
                scroll: 0,
                visible: 0,
            },
            HelpKind::TUI { .. } => HelpKind::Service {
                scroll: 0,
                visible: 0,
            },
            HelpKind::Service { .. } => HelpKind::None,
        };
    }

    pub fn scroll_down(&mut self, service_doc: &str) {
        let (content, scroll, visible) = match self {
            HelpKind::TUI { scroll, visible } => {
                (tui_help_text().lines.len() as u16, scroll, *visible)
            }
            HelpKind::Service { scroll, visible } => {
                (service_doc.lines().count() as u16, scroll, *visible)
            }
            HelpKind::None => return,
        };
        let max_scroll = content.saturating_sub(visible);
        *scroll = (*scroll + 1).min(max_scroll);
    }

    pub fn scroll_up(&mut self) {
        match self {
            HelpKind::TUI { scroll, .. } | HelpKind::Service { scroll, .. } => {
                *scroll = scroll.saturating_sub(1);
            }
            HelpKind::None => {}
        }
    }

    pub fn draw_help_pop_up(&mut self, frame: &mut Frame, service_doc: &str) {
        let (text, scroll): (Option<Text>, u16) = match self {
            HelpKind::None => (None, 0),
            HelpKind::TUI { scroll, .. } => (Some(tui_help_text()), *scroll),
            HelpKind::Service { scroll, .. } => (Some(Text::raw(service_doc.to_string())), *scroll),
        };
        if let Some(help_text) = text {
            let popup_area = make_popup_area(frame.area(), 80, 80);
            // store visible height for scroll capping (subtract 2 for the border)
            let inner_height = popup_area.height.saturating_sub(2);
            match self {
                HelpKind::TUI { visible, .. } | HelpKind::Service { visible, .. } => {
                    *visible = inner_height;
                }
                HelpKind::None => {}
            }
            frame.render_widget(Clear, popup_area);
            let paragraph = Paragraph::new(help_text)
                .block(Block::bordered().title("Ledgera TUI help  ↑/↓ to scroll:"))
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false })
                .scroll((scroll, 0));
            frame.render_widget(paragraph, popup_area);
        }
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn make_popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
