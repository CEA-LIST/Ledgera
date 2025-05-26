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

use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::Frame;

use crate::app::LedgeraTuiInputMode;
use crate::commands::command_text_edition::LedgeraTuiCommandEditorState;

pub fn render_input_area(
    input_mode: &LedgeraTuiInputMode,
    frame: &mut Frame,
    input_area: Rect,
    command_editor: &LedgeraTuiCommandEditorState,
) {
    let border_style = match input_mode {
        LedgeraTuiInputMode::Editing => Style::default().fg(Color::Yellow),
        LedgeraTuiInputMode::Browsing => Style::default().fg(Color::Magenta),
        LedgeraTuiInputMode::MainMenu => Style::default(),
    };
    let input = ratatui::widgets::Paragraph::new(command_editor.get_input())
        .style(match input_mode {
            LedgeraTuiInputMode::Editing => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        })
        .block(
            ratatui::widgets::Block::bordered()
                .title(Span::styled(
                    "Command input",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .border_style(border_style),
        );
    frame.render_widget(input, input_area);
    if input_mode == &LedgeraTuiInputMode::Editing {
        frame.set_cursor_position(Position::new(
            // Draw the cursor at the current position in the input field.
            input_area.x + command_editor.get_character_index() as u16 + 1,
            // Move one line down, from the border to the input line
            input_area.y + 1,
        ));
    }
}
