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

//! The [`LedgeraTuiBehavior`] trait isolates everything that is *app-specific* in a Ledgera TUI,
//! so that the generic event loop / rendering / knowledge management can live once in
//! [`crate::engine::LedgeraTui`].
//!
//! What varies between apps (and is captured here):
//!  - the command type and how a raw command line is parsed into it;
//!  - how a parsed command is handled — whether it drives the core runtime directly
//!    (the "requests == function instances" case, see [`crate::behaviors::function_instance`]),
//!    sends a high-level request to an app service client, or runs a purely local effect;
//!  - what *backend events* the app reacts to in the background (core feedback, and any
//!    app-specific channels);
//!  - any extra UI pane and the state behind it.

use ledgera_node_client::comms::feedback_from_core_client::ValidatedCoreFeedbackMessage;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;

use crate::knowledge::tui_knowledge::LedgeraTuiKnowledge;

/// What the engine should do after a command has been handled.
pub enum TuiControlFlow {
    /// stay in the current input mode
    Continue,
    /// leave the editing mode and go back to the main menu (e.g. an `exit` command)
    ReturnToMenu,
}

/// A background event surfaced by a behavior's backend.
pub enum TuiBackgroundEvent<LAT: LedgeraApplicationTemplate> {
    /// validated feedback from the Ledgera core, to be folded into the shared knowledge
    CoreFeedback(ValidatedCoreFeedbackMessage<LAT>),
    /// the behavior handled an app-specific event itself (e.g. updated its own state);
    /// the engine just needs to redraw
    Redraw,
}

/// App-specific behavior plugged into the generic [`crate::engine::LedgeraTui`] engine.
pub trait LedgeraTuiBehavior {
    /// the Ledgera application template this TUI is built for
    type App: LedgeraApplicationTemplate;
    /// the app-specific command type produced by [`Self::parse_command`]
    type Command;

    /// Documentation shown in the "service" help overlay (toggled with 'o').
    /// Default: empty.
    fn service_doc(&self) -> &'static str {
        ""
    }

    /// Parse a raw command line into a command, or return a human-readable error
    /// (shown in the command history).
    fn parse_command(input: &str) -> Result<Self::Command, String>;

    /// Validate any monikers the command references against current knowledge.
    /// Default: accept everything.
    fn check_monikers(
        _cmd: &Self::Command,
        _knowledge: &LedgeraTuiKnowledge<Self::App>,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Execute a parsed command. The behavior may talk to its own backend, mutate its
    /// own state, and read/refresh `knowledge`.
    async fn handle_command(
        &mut self,
        cmd: Self::Command,
        knowledge: &mut LedgeraTuiKnowledge<Self::App>,
        node_name: &str,
    ) -> TuiControlFlow;

    /// Await the next background event. Core feedback is returned for the engine to fold
    /// into `knowledge`; app-specific events are handled internally and reported as
    /// [`TuiBackgroundEvent::Redraw`]. Implementations that have no more events should
    /// await a never-resolving future (`std::future::pending().await`) so the engine's
    /// `select!` simply relies on its other branches instead of busy-looping.
    async fn next_background_event(&mut self) -> TuiBackgroundEvent<Self::App>;

    /// Height (in rows) of an optional extra pane rendered just above the log area.
    /// Default `0` means no extra pane.
    fn app_pane_height(&self) -> u16 {
        0
    }

    /// Render the optional extra pane (only called when [`Self::app_pane_height`] > 0).
    fn render_app_pane(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
}
