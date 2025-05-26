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

//! Command-type-agnostic command line: text editor + command history with Up/Down
//! navigation. Parsing is left to the [`crate::behavior::LedgeraTuiBehavior`]; this only
//! deals in raw strings and a `(display, ok)` history.

use crate::commands::command_text_edition::LedgeraTuiCommandEditorState;

pub struct TuiCommandLine {
    /// state of the text edition area for inputting commands
    editor: LedgeraTuiCommandEditorState,
    /// history of submitted commands, with whether each was accepted
    history: Vec<(String, bool)>,
    /// index into `history` while navigating with Up/Down; `None` = not browsing
    history_cursor: Option<usize>,
    /// draft saved when the user first presses Up, restored when they come back down
    history_draft: String,
}

impl Default for TuiCommandLine {
    fn default() -> Self {
        Self::new()
    }
}

impl TuiCommandLine {
    pub fn new() -> Self {
        Self {
            editor: LedgeraTuiCommandEditorState::new(),
            history: Vec::new(),
            history_cursor: None,
            history_draft: String::new(),
        }
    }

    pub fn get_editor(&self) -> &LedgeraTuiCommandEditorState {
        &self.editor
    }

    pub fn get_commands_history(&self) -> &Vec<(String, bool)> {
        &self.history
    }

    pub fn handle_key_pressed_event(&mut self, key_code: ratatui::crossterm::event::KeyCode) {
        self.editor.handle_key_pressed_event(key_code);
        // any edit (other than history navigation) breaks out of history browsing
        match key_code {
            ratatui::crossterm::event::KeyCode::Up | ratatui::crossterm::event::KeyCode::Down => {}
            _ => {
                self.history_cursor = None;
            }
        }
    }

    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let next = match self.history_cursor {
            None => {
                self.history_draft = self.editor.get_input().to_string();
                self.history.len() - 1
            }
            Some(0) => 0,
            Some(i) => i - 1,
        };
        self.history_cursor = Some(next);
        let entry = self.history[next].0.clone();
        self.editor.set_input(&entry);
    }

    pub fn history_down(&mut self) {
        match self.history_cursor {
            None => {}
            Some(i) if i + 1 >= self.history.len() => {
                self.history_cursor = None;
                let draft = self.history_draft.clone();
                self.editor.set_input(&draft);
            }
            Some(i) => {
                let next = i + 1;
                self.history_cursor = Some(next);
                let entry = self.history[next].0.clone();
                self.editor.set_input(&entry);
            }
        }
    }

    /// Clear the input box and return what was typed, resetting history browsing.
    pub fn take_input(&mut self) -> String {
        self.history_cursor = None;
        self.history_draft.clear();
        self.editor.get_message_and_clear_input()
    }

    /// Record an entry in the history (`display` is what is shown, `ok` controls styling).
    pub fn record(&mut self, display: String, ok: bool) {
        self.history.push((display, ok));
    }
}
