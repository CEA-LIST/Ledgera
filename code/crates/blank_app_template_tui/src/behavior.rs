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

//! The blank-template [`LedgeraTuiBehavior`] — the canonical starting point for a concrete
//! service's TUI.
//!
//! Concrete services derive their TUI from this file by:
//!  1. adding command variants in [`crate::commands::tui_commands`] and parsing them in
//!     [`crate::commands::parse_command`];
//!  2. dispatching each new command in [`ServiceTemplateBehavior::handle_command`] via
//!     `self.service_client_runtime_io.user_requests_sender`;
//!  3. (optionally) overriding [`LedgeraTuiBehavior::app_pane_height`] /
//!     [`LedgeraTuiBehavior::render_app_pane`] to show app-specific state.
//!
//! Everything else — the event loop, input modes, the knowledge cache fed by validated core
//! feedback, the JSON knowledge explorer, the help overlay and command history
//! is provided once by [`ledgera_util_basic_tui::engine`].

use std::collections::BTreeSet;

use ledgera_blank_app_template::lat_binding::LedgeraServiceTemplate;
use ledgera_blank_app_template::service_client_logic::runtime_io::ServiceClientRuntimeIO;
use ledgera_util_basic_tui::behavior::{LedgeraTuiBehavior, TuiBackgroundEvent, TuiControlFlow};
use ledgera_util_basic_tui::knowledge::tui_knowledge::LedgeraTuiKnowledge;

use crate::commands::parse_command::parse_ledgera_service_template_tui_command;
use crate::commands::tui_commands::LedgeraServiceTemplateTuiCommand;

const SERVICE_TEMPLATE_DOC: &str = "\
Service template commands:
  exit
  (add your own commands in commands/tui_commands.rs + commands/parse_command.rs,
   then dispatch them in behavior.rs::handle_command)
";

pub struct ServiceTemplateBehavior {
    /// channels to/from the service-client backend. The whole struct is kept (rather than
    /// destructured) so that `user_requests_sender` stays available for concrete services to wire
    /// their commands to, without tripping the dead-code lint while the template has none.
    service_client_runtime_io: ServiceClientRuntimeIO,
    /// Static list of known service-client identities (loaded from the PKI folder at start-up),
    /// surfaced in the app pane as an example of [`LedgeraTuiBehavior::render_app_pane`].
    all_service_clients_names: BTreeSet<String>,
}

impl ServiceTemplateBehavior {
    pub fn new(
        service_client_runtime_io: ServiceClientRuntimeIO,
        all_service_clients_names: BTreeSet<String>,
    ) -> Self {
        Self {
            service_client_runtime_io,
            all_service_clients_names,
        }
    }
}

impl LedgeraTuiBehavior for ServiceTemplateBehavior {
    type App = LedgeraServiceTemplate;
    type Command = LedgeraServiceTemplateTuiCommand;

    fn service_doc(&self) -> &'static str {
        SERVICE_TEMPLATE_DOC
    }

    fn parse_command(input: &str) -> Result<Self::Command, String> {
        parse_ledgera_service_template_tui_command(input)
    }

    async fn handle_command(
        &mut self,
        cmd: Self::Command,
        _knowledge: &mut LedgeraTuiKnowledge<Self::App>,
        _node_name: &str,
    ) -> TuiControlFlow {
        match cmd {
            LedgeraServiceTemplateTuiCommand::Exit => return TuiControlFlow::ReturnToMenu,
            // TODO : when concrete services add command variants, dispatch them here, e.g.
            //   LedgeraServiceTemplateTuiCommand::SomeAction(args) => {
            //       let _ = self
            //           .service_client_runtime_io
            //           .user_requests_sender
            //           .send(HighLevelServiceUserRequests::SomeAction(args))
            //           .await;
            //   }
        }
        TuiControlFlow::Continue
    }

    async fn next_background_event(&mut self) -> TuiBackgroundEvent<Self::App> {
        // Forward validated core feedback to the engine, which folds it into the shared
        // knowledge cache.
        match self
            .service_client_runtime_io
            .validated_core_msgs_receiver
            .recv()
            .await
        {
            Some(feedback) => TuiBackgroundEvent::CoreFeedback(feedback),
            None => std::future::pending().await,
        }
    }

    fn app_pane_height(&self) -> u16 {
        6
    }

    fn render_app_pane(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let block = ratatui::widgets::Block::default()
            .title("Known service clients")
            .borders(ratatui::widgets::Borders::ALL);
        let lines: Vec<ratatui::text::Line> = self
            .all_service_clients_names
            .iter()
            .map(|name| ratatui::text::Line::from(name.as_str()))
            .collect();
        let paragraph = ratatui::widgets::Paragraph::new(lines)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }
}
