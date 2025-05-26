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

pub struct LedgeraTuiCommandEditorState {
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    character_index: usize,
}

impl Default for LedgeraTuiCommandEditorState {
    fn default() -> Self {
        Self::new()
    }
}

impl LedgeraTuiCommandEditorState {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            character_index: 0,
        }
    }

    pub fn get_input(&self) -> &str {
        self.input.as_str()
    }

    pub fn get_character_index(&self) -> usize {
        self.character_index
    }

    pub fn handle_key_pressed_event(&mut self, key_code: ratatui::crossterm::event::KeyCode) {
        match key_code {
            ratatui::crossterm::event::KeyCode::Char(to_insert) => self.enter_char(to_insert),
            ratatui::crossterm::event::KeyCode::Backspace => self.delete_char(),
            ratatui::crossterm::event::KeyCode::Left => self.move_cursor_left(),
            ratatui::crossterm::event::KeyCode::Right => self.move_cursor_right(),
            _ => {}
        }
    }

    pub fn get_message_and_clear_input(&mut self) -> String {
        let message = self.input.clone();
        self.input.clear();
        self.reset_cursor();
        message
    }

    pub fn set_input(&mut self, s: &str) {
        self.input = s.to_string();
        self.character_index = self.input.chars().count();
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }
}
