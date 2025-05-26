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

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem};
use ratatui::Frame;

use crate::app::LedgeraTuiInputMode;

pub fn render_info_area(
    input_mode: &LedgeraTuiInputMode,
    frame: &mut Frame,
    info_area: Rect,
    history: &[(String, bool)],
) -> Rect {
    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(info_area);

    let border_style = match input_mode {
        LedgeraTuiInputMode::Editing => Style::default().fg(Color::Yellow),
        LedgeraTuiInputMode::Browsing => Style::default().fg(Color::Magenta),
        LedgeraTuiInputMode::MainMenu => Style::default(),
    };

    let messages = {
        let messages: Vec<ListItem> = history
            .iter()
            .enumerate()
            .map(|(i, (request_string, is_valid))| {
                let mut line = Line::from(Span::raw(format!("{}:{}", i, request_string)));
                if *is_valid {
                    line = line.patch_style(Style::default().fg(Color::Green));
                } else {
                    line = line.patch_style(Style::default().fg(Color::Red));
                }
                ListItem::new(line)
            })
            .collect();
        List::new(messages).block(
            Block::bordered()
                .title(Span::styled(
                    "User requests history",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ))
                .border_style(border_style),
        )
    };
    frame.render_widget(messages, inner_layout[0]);

    inner_layout[1]
}
