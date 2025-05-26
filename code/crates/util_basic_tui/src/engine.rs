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

//! Generic Ledgera TUI engine: owns the event loop, input modes, knowledge, the JSON
//! knowledge explorer, the help overlay and the rendering skeleton. Everything
//! app-specific is delegated to a [`LedgeraTuiBehavior`].

use futures::{FutureExt, StreamExt};
use ledgera_knowledge_representation::json::get_ledgera_knowledge_json_representation;

use crate::app::LedgeraTuiInputMode;
use crate::behavior::{LedgeraTuiBehavior, TuiBackgroundEvent, TuiControlFlow};
use crate::command_line::TuiCommandLine;
use crate::help::HelpKind;
use crate::json_explorer::JsonExplorer;
use crate::knowledge::tui_knowledge::LedgeraTuiKnowledge;
use crate::logger::MyLogFormatter;
use crate::rendering::area_help::render_help_area;
use crate::rendering::area_info::render_info_area;
use crate::rendering::area_input::render_input_area;

/// The generic TUI engine, parameterized by an app-specific [`LedgeraTuiBehavior`].
pub struct LedgeraTui<B: LedgeraTuiBehavior> {
    /// name of the node the TUI and underlying client is co-located with
    node_name: String,
    /// current input mode
    input_mode: LedgeraTuiInputMode,
    /// text input + command history
    command_line: TuiCommandLine,
    /// app-specific behavior (backend + command handling + extra UI)
    behavior: B,
    /// whether the application is running
    running: bool,
    /// state of the "Help" overlay
    help_state: HelpKind,
    /// crossterm event stream
    event_stream: ratatui::crossterm::event::EventStream,
    /// the knowledge the TUI has about the Ledgera system
    knowledge: LedgeraTuiKnowledge<B::App>,
    /// JSON encoding of the knowledge (refreshed every draw)
    json_knowledge_repr: serde_json::Value,
    /// widget representing that JSON encoding
    json_explorer: JsonExplorer,
}

impl<B: LedgeraTuiBehavior> LedgeraTui<B> {
    pub fn new(node_name: String, behavior: B) -> Self {
        Self {
            node_name,
            input_mode: LedgeraTuiInputMode::MainMenu,
            command_line: TuiCommandLine::new(),
            behavior,
            running: false,
            help_state: HelpKind::None,
            event_stream: ratatui::crossterm::event::EventStream::default(),
            knowledge: LedgeraTuiKnowledge::new(),
            json_knowledge_repr: serde_json::Value::Null,
            json_explorer: JsonExplorer::new(),
        }
    }

    pub async fn run(mut self, mut terminal: ratatui::DefaultTerminal) {
        self.running = true;
        terminal.draw(|frame| self.draw(frame)).unwrap();
        while self.running {
            self.listen_and_update().await;
            terminal.draw(|frame| self.draw(frame)).unwrap();
        }
    }

    async fn listen_and_update(&mut self) {
        tokio::select! {
            // terminal events
            event = self.event_stream.next().fuse() => {
                if let Some(Ok(ratatui::crossterm::event::Event::Key(key))) = event {
                    self.handle_key_event_on_terminal(key).await;
                }
            },
            // background events from the behavior's backend
            bg = self.behavior.next_background_event() => {
                match bg {
                    TuiBackgroundEvent::CoreFeedback(feedback) => {
                        self.knowledge.update_with_core_client_feedback(feedback);
                    }
                    TuiBackgroundEvent::Redraw => {}
                }
            }
            // otherwise tick to avoid busy waiting
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {}
        }
    }

    async fn process_command(&mut self) {
        let raw = self.command_line.take_input();
        match B::parse_command(&raw) {
            Ok(cmd) => match B::check_monikers(&cmd, &self.knowledge) {
                Ok(()) => {
                    self.command_line.record(raw, true);
                    match self
                        .behavior
                        .handle_command(cmd, &mut self.knowledge, &self.node_name)
                        .await
                    {
                        TuiControlFlow::ReturnToMenu => {
                            self.input_mode = LedgeraTuiInputMode::MainMenu;
                        }
                        TuiControlFlow::Continue => {}
                    }
                }
                Err(e) => self.command_line.record(format!("{} | {}", raw, e), false),
            },
            Err(e) => self.command_line.record(format!("{} | {}", raw, e), false),
        }
    }

    async fn handle_key_event_on_terminal(&mut self, key: ratatui::crossterm::event::KeyEvent) {
        if key.kind != ratatui::crossterm::event::KeyEventKind::Press {
            return;
        }
        use ratatui::crossterm::event::KeyCode;
        match self.input_mode {
            LedgeraTuiInputMode::MainMenu => match key.code {
                KeyCode::Char('e') => {
                    self.help_state = HelpKind::None;
                    self.input_mode = LedgeraTuiInputMode::Editing;
                }
                KeyCode::Char('b') => {
                    self.help_state = HelpKind::None;
                    self.input_mode = LedgeraTuiInputMode::Browsing;
                }
                KeyCode::Char('q') => {
                    self.running = false;
                }
                KeyCode::Char('h') => self.help_state.h_pressed(),
                KeyCode::Char('o') => self.help_state.o_pressed(),
                KeyCode::Down if self.help_state.is_open() => {
                    self.help_state.scroll_down(self.behavior.service_doc());
                }
                KeyCode::Up if self.help_state.is_open() => {
                    self.help_state.scroll_up();
                }
                _ => {}
            },
            LedgeraTuiInputMode::Browsing => match key.code {
                KeyCode::Char('q') => {
                    self.input_mode = LedgeraTuiInputMode::MainMenu;
                }
                KeyCode::Down => self.json_explorer.on_down(&self.json_knowledge_repr),
                KeyCode::Up => self.json_explorer.on_up(&self.json_knowledge_repr),
                KeyCode::Left => self.json_explorer.on_left(),
                KeyCode::Right => self.json_explorer.on_right(&self.json_knowledge_repr),
                _ => {}
            },
            LedgeraTuiInputMode::Editing => match key.code {
                KeyCode::Enter => self.process_command().await,
                KeyCode::Up => self.command_line.history_up(),
                KeyCode::Down => self.command_line.history_down(),
                _ => self.command_line.handle_key_pressed_event(key.code),
            },
        }
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        let vertical = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Min(1),
            ratatui::layout::Constraint::Min(1),
        ]);
        let [help_area, input_area, info_area, bottom_area] = vertical.areas(frame.area());

        render_help_area(&self.input_mode, frame, help_area);
        render_input_area(
            &self.input_mode,
            frame,
            input_area,
            self.command_line.get_editor(),
        );

        self.json_knowledge_repr = get_ledgera_knowledge_json_representation(
            &self.knowledge.cached_client_knowledge,
            &self
                .knowledge
                .data_monikers
                .clone()
                .into_iter()
                .map(|(k, v)| (v, k))
                .collect(),
            &self
                .knowledge
                .computations_monikers
                .clone()
                .into_iter()
                .map(|(k, v)| (v, k))
                .collect(),
        );
        let explorer_area = render_info_area(
            &self.input_mode,
            frame,
            info_area,
            self.command_line.get_commands_history(),
        );
        self.json_explorer
            .render(frame, explorer_area, &self.json_knowledge_repr);

        // bottom region: optional app pane above the logs
        let app_pane_height = self.behavior.app_pane_height();
        let log_area = if app_pane_height > 0 {
            let split = ratatui::layout::Layout::vertical([
                ratatui::layout::Constraint::Length(app_pane_height),
                ratatui::layout::Constraint::Min(1),
            ])
            .split(bottom_area);
            self.behavior.render_app_pane(frame, split[0]);
            split[1]
        } else {
            bottom_area
        };

        let log_block = ratatui::widgets::Block::default()
            .title(ratatui::text::Span::styled(
                "Ledgera Node Logs",
                ratatui::style::Style::default()
                    .fg(ratatui::style::Color::LightRed)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ))
            .borders(ratatui::widgets::Borders::ALL);
        let log_widget = tui_logger::TuiLoggerWidget::default()
            .formatter(Box::new(MyLogFormatter {}))
            .block(log_block);
        frame.render_widget(log_widget, log_area);

        self.help_state
            .draw_help_pop_up(frame, self.behavior.service_doc());
    }
}
