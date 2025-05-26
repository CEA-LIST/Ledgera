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

//! The Varkeep-specific [`LedgeraTuiBehavior`]: a small `locassign`/`gloassign` command set, a
//! high-level request channel to the Varkeep service client, and a "variables" pane fed by the
//! service client's `(name, value)` updates.
//!
//! Unlike the function-instance behavior, Varkeep talks to a [`ServiceClientRuntimeIO`] which
//! already consumes the raw core feedback internally; the TUI therefore never receives
//! [`TuiBackgroundEvent::CoreFeedback`] and the shared knowledge graph stays empty.

use std::collections::BTreeMap;

use ledgera_app_varkeep::lat_binding::LedgeraVarkeepService;
use ledgera_app_varkeep::service_client_logic::runtime_io::ServiceClientRuntimeIO;
use ledgera_app_varkeep::service_client_logic::user_reqs::HighLevelVarkeepUserRequests;
use ledgera_util_basic_tui::behavior::{LedgeraTuiBehavior, TuiBackgroundEvent, TuiControlFlow};
use ledgera_util_basic_tui::knowledge::tui_knowledge::LedgeraTuiKnowledge;

use crate::commands::parse_command::parse_ledgera_varkeep_tui_command;
use crate::commands::tui_commands::LedgeraVarkeepServiceTuiCommand;

const VARKEEP_SERVICE_DOC: &str = "\
Varkeep commands:
  locassign <name> <value>   assign <value> to the node-local variable <name>
  gloassign <name> <value>   assign <value> to the globally-anchored variable <name>
  exit
";

pub struct VarkeepBehavior {
    /// to send high-level requests to the Varkeep service client backend
    user_requests_sender: tokio::sync::mpsc::Sender<HighLevelVarkeepUserRequests>,
    /// receives `(variable_name, variable_value)` updates from the service client
    tui_feed_receiver: tokio::sync::mpsc::Receiver<(String, String)>,
    /// the variables known to the TUI (rendered in the app pane)
    varmap: BTreeMap<String, String>,
}

impl VarkeepBehavior {
    pub fn new(service_client_runtime_io: ServiceClientRuntimeIO) -> Self {
        Self {
            user_requests_sender: service_client_runtime_io.user_requests_sender,
            tui_feed_receiver: service_client_runtime_io.tui_feed_receiver,
            varmap: BTreeMap::new(),
        }
    }

    async fn send_request(&self, request: HighLevelVarkeepUserRequests, label: &str) {
        if let Err(e) = self.user_requests_sender.send(request).await {
            log::warn!("TUI : could not send {} user request: {:?}", label, e);
        } else {
            log::info!("TUI : sent {} to the Varkeep service client", label);
        }
    }
}

impl LedgeraTuiBehavior for VarkeepBehavior {
    type App = LedgeraVarkeepService;
    type Command = LedgeraVarkeepServiceTuiCommand;

    fn service_doc(&self) -> &'static str {
        VARKEEP_SERVICE_DOC
    }

    fn parse_command(input: &str) -> Result<Self::Command, String> {
        parse_ledgera_varkeep_tui_command(input)
    }

    async fn handle_command(
        &mut self,
        cmd: Self::Command,
        _knowledge: &mut LedgeraTuiKnowledge<Self::App>,
        _node_name: &str,
    ) -> TuiControlFlow {
        match cmd {
            LedgeraVarkeepServiceTuiCommand::Exit => return TuiControlFlow::ReturnToMenu,
            LedgeraVarkeepServiceTuiCommand::AssignLocal(vn, vv) => {
                self.send_request(
                    HighLevelVarkeepUserRequests::AssignLocal(vn, vv),
                    "AssignLocal",
                )
                .await;
            }
            LedgeraVarkeepServiceTuiCommand::AssignGlobal(vn, vv) => {
                self.send_request(
                    HighLevelVarkeepUserRequests::AssignGlobal(vn, vv),
                    "AssignGlobal",
                )
                .await;
            }
        }
        TuiControlFlow::Continue
    }

    async fn next_background_event(&mut self) -> TuiBackgroundEvent<Self::App> {
        match self.tui_feed_receiver.recv().await {
            Some((vn, vv)) => {
                self.varmap.insert(vn, vv);
                TuiBackgroundEvent::Redraw
            }
            None => std::future::pending().await,
        }
    }

    fn app_pane_height(&self) -> u16 {
        10
    }

    fn render_app_pane(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let block = ratatui::widgets::Block::default()
            .title("Variables")
            .borders(ratatui::widgets::Borders::ALL);
        let lines: Vec<ratatui::text::Line> = self
            .varmap
            .iter()
            .map(|(k, v)| ratatui::text::Line::from(format!("{} = {};", k, v)))
            .collect();
        let paragraph = ratatui::widgets::Paragraph::new(lines)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }
}
