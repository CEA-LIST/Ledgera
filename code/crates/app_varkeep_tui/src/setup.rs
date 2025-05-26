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

use crate::behavior::VarkeepBehavior;
use ledgera_app_varkeep::service_client_logic::runtime_io::ServiceClientRuntimeIO;
use ledgera_util_basic_tui::engine::LedgeraTui;
use log::LevelFilter;
use tui_logger::{init_logger, set_default_level};

pub fn ledgera_tui_log_setup() {
    color_eyre::install().unwrap();
    init_logger(LevelFilter::Info).unwrap();
    set_default_level(LevelFilter::Info);
}

pub async fn ledgera_service_tui_setup(
    node_name: String,
    service_client_runtime_io: ServiceClientRuntimeIO,
) {
    let terminal = ratatui::init();
    let behavior = VarkeepBehavior::new(service_client_runtime_io);
    LedgeraTui::new(node_name, behavior).run(terminal).await;
    ratatui::restore();
}
