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

use ledgera_node_client::io::parser::LedgeraComputationItemsParser;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;

use crate::commands::command_text_edition::LedgeraTuiCommandEditorState;
use crate::commands::parse_command::parse_ledgera_tui_command;
use crate::commands::tui_commands::LedgeraTuiCommand;
use crate::knowledge::tui_knowledge::LedgeraTuiKnowledge;
use std::marker::PhantomData;

pub struct LedgeraTuiCommandsManager<
    LAT: LedgeraApplicationTemplate,
    CmpParser: LedgeraComputationItemsParser<LAT>,
> {
    /// state of the text edition area for inputing Ledgera commands
    command_editor_state: LedgeraTuiCommandEditorState,
    /// History of valid user requests
    commands_history: Vec<(String, bool)>,
    /// Index into commands_history when navigating with Up/Down; None = not browsing
    history_cursor: Option<usize>,
    /// Draft saved when the user first presses Up, restored when they come back down
    history_draft: String,
    phantom: PhantomData<LAT>,
    phantom2: PhantomData<CmpParser>,
}

impl<LAT: LedgeraApplicationTemplate, CmpParser: LedgeraComputationItemsParser<LAT>> Default
    for LedgeraTuiCommandsManager<LAT, CmpParser>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<LAT: LedgeraApplicationTemplate, CmpParser: LedgeraComputationItemsParser<LAT>>
    LedgeraTuiCommandsManager<LAT, CmpParser>
{
    pub fn new() -> Self {
        Self {
            command_editor_state: LedgeraTuiCommandEditorState::new(),
            commands_history: Vec::new(),
            history_cursor: None,
            history_draft: String::new(),
            phantom: PhantomData,
            phantom2: PhantomData,
        }
    }

    pub fn get_editor(&self) -> &LedgeraTuiCommandEditorState {
        &self.command_editor_state
    }

    pub fn get_commands_history(&self) -> &Vec<(String, bool)> {
        &self.commands_history
    }

    pub fn handle_key_pressed_event(&mut self, key_code: ratatui::crossterm::event::KeyCode) {
        self.command_editor_state.handle_key_pressed_event(key_code);
        // any edit while browsing history breaks out of history navigation
        match key_code {
            ratatui::crossterm::event::KeyCode::Up | ratatui::crossterm::event::KeyCode::Down => {}
            _ => {
                self.history_cursor = None;
            }
        }
    }

    pub fn history_up(&mut self) {
        if self.commands_history.is_empty() {
            return;
        }
        let next = match self.history_cursor {
            None => {
                // save current draft before entering history
                self.history_draft = self.command_editor_state.get_input().to_string();
                self.commands_history.len() - 1
            }
            Some(0) => 0,
            Some(i) => i - 1,
        };
        self.history_cursor = Some(next);
        let entry = self.commands_history[next].0.clone();
        self.command_editor_state.set_input(&entry);
    }

    pub fn history_down(&mut self) {
        match self.history_cursor {
            None => {}
            Some(i) if i + 1 >= self.commands_history.len() => {
                // past the end — restore draft
                self.history_cursor = None;
                let draft = self.history_draft.clone();
                self.command_editor_state.set_input(&draft);
            }
            Some(i) => {
                let next = i + 1;
                self.history_cursor = Some(next);
                let entry = self.commands_history[next].0.clone();
                self.command_editor_state.set_input(&entry);
            }
        }
    }

    pub fn submit_command(
        &mut self,
        k: &LedgeraTuiKnowledge<LAT>,
    ) -> Option<LedgeraTuiCommand<LAT>> {
        self.history_cursor = None;
        self.history_draft.clear();
        let raw_command_string = self.command_editor_state.get_message_and_clear_input();
        // ***
        match parse_ledgera_tui_command::<LAT, CmpParser>(&raw_command_string) {
            Ok(cmd) => match cmd.check_monikers(k) {
                Ok(_) => {
                    self.commands_history
                        .push((raw_command_string.to_string(), true));
                    Some(cmd)
                }
                Err(e) => {
                    self.commands_history
                        .push((format!("{} | {:?}", raw_command_string, e), false));
                    None
                }
            },
            Err(e) => {
                self.commands_history
                    .push((format!("{} | {}", raw_command_string, e), false));
                None
            }
        }
    }
}
