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

use std::collections::BTreeSet;

use ledgera_blank_app_template::service_client_logic::runtime_io::ServiceClientRuntimeIO;
use ledgera_util_basic_tui::engine::LedgeraTui;
use log::LevelFilter;
use tui_logger::{init_logger, set_default_level};

use crate::behavior::ServiceTemplateBehavior;

/// Call this BEFORE starting the node when the local node is going to be a client (so that
/// log output is captured by the in-TUI log pane instead of polluting the ratatui screen).
pub fn ledgera_tui_log_setup() {
    color_eyre::install().unwrap();
    init_logger(LevelFilter::Info).unwrap();
    set_default_level(LevelFilter::Info);
}

/// Call this AFTER `LedgeraServiceClientState::run()` has returned its `ServiceClientRuntimeIO`.
/// Takes control of the terminal, drives the TUI loop, then restores the terminal on exit.
pub async fn ledgera_service_tui_setup(
    node_name: String,
    service_client_runtime_io: ServiceClientRuntimeIO,
    all_service_clients_names: BTreeSet<String>,
) {
    let terminal = ratatui::init();
    let behavior =
        ServiceTemplateBehavior::new(service_client_runtime_io, all_service_clients_names);
    LedgeraTui::new(node_name, behavior).run(terminal).await;
    ratatui::restore();
}
