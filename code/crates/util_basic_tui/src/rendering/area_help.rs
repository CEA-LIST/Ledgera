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

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::Frame;

use crate::app::LedgeraTuiInputMode;

pub fn render_help_area(input_mode: &LedgeraTuiInputMode, frame: &mut Frame, help_area: Rect) {
    let (mode_label, key_color) = match input_mode {
        LedgeraTuiInputMode::MainMenu => ("[MENU] ", Color::Cyan),
        LedgeraTuiInputMode::Editing => ("[EDIT] ", Color::Yellow),
        LedgeraTuiInputMode::Browsing => ("[BROWSE] ", Color::Magenta),
    };
    let key_style = Style::default().fg(key_color).add_modifier(Modifier::BOLD);
    let mut line = Line::from(Span::styled(mode_label, key_style));
    line.push_span(Span::raw("Press "));
    match input_mode {
        LedgeraTuiInputMode::MainMenu => {
            line.push_span(Span::styled("e", key_style));
            line.push_span(Span::raw(" to go to EDITION mode, "));
            line.push_span(Span::styled("b", key_style));
            line.push_span(Span::raw(" to go to BROWSING mode, "));
            line.push_span(Span::styled("q", key_style));
            line.push_span(Span::raw(" to exit session and terminate node, "));
            line.push_span(Span::styled("h", key_style));
            line.push_span(Span::raw("/"));
            line.push_span(Span::styled("o", key_style));
            line.push_span(Span::raw(" to display HELP"));
        }
        LedgeraTuiInputMode::Editing => {
            line.push_span(Span::styled("ENTER", key_style));
            line.push_span(Span::raw(
                " to send a command (\"exit\" to exit EDITION mode)",
            ));
        }
        LedgeraTuiInputMode::Browsing => {
            line.push_span(Span::styled("q", key_style));
            line.push_span(Span::raw(" to exit BROWSING mode and "));
            line.push_span(Span::styled("←", key_style));
            line.push_span(Span::raw(" / "));
            line.push_span(Span::styled("→", key_style));
            line.push_span(Span::raw(" / "));
            line.push_span(Span::styled("↑", key_style));
            line.push_span(Span::raw(" / "));
            line.push_span(Span::styled("↓", key_style));
            line.push_span(Span::raw(" to navigate"));
        }
    }
    let text = ratatui::text::Text::from(line);
    let help_message = ratatui::widgets::Paragraph::new(text);
    frame.render_widget(help_message, help_area);
}
