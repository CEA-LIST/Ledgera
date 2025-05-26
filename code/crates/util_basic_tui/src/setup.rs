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

use crate::behaviors::function_instance::FunctionInstanceBehavior;
use crate::doc::DocumentedCmpTying;
use crate::engine::LedgeraTui;
use ledgera_comms::comm_session::PubSubNetwork;
use ledgera_knowledge_representation::printer::LedgeraComputationItemsPrinter;
use ledgera_node_client::{
    client_logic::client_behavior::LedgeraClientRunOutput,
    io::parser::LedgeraComputationItemsParser,
};
use ledgera_pki::manager::PublicKeyInfrastructure;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use log::LevelFilter;
use tui_logger::{init_logger, set_default_level};

pub fn ledgera_tui_log_setup() {
    color_eyre::install().unwrap();
    init_logger(LevelFilter::Info).unwrap();
    set_default_level(LevelFilter::Info);
}

/// Run the TUI for an app whose commands map directly onto core function instances.
/// This is the generic engine driven by the provided [`FunctionInstanceBehavior`].
pub async fn ledgera_tui_setup<
    PKI: PublicKeyInfrastructure,
    Sess: PubSubNetwork,
    LAT: LedgeraApplicationTemplate + DocumentedCmpTying,
    CmpParser: LedgeraComputationItemsParser<LAT>,
    CmpPrinter: LedgeraComputationItemsPrinter<LAT>,
>(
    node_name: String,
    client_node_run: LedgeraClientRunOutput<PKI, Sess, LAT>,
) {
    let terminal = ratatui::init();
    let behavior =
        FunctionInstanceBehavior::<PKI, Sess, LAT, CmpParser, CmpPrinter>::new(client_node_run);
    LedgeraTui::new(node_name, behavior).run(terminal).await;
    ratatui::restore();
}
