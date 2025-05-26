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

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use ledgera_knowledge_representation::know::LedgeraKnowledgeRepresentation;
use ledgera_knowledge_representation::print_graph_v3::main_graph::print_current_knowledge_as_graph_v3;
use ledgera_knowledge_representation::printer::LedgeraComputationItemsPrinter;
use ledgera_pki::manager::SerdeSerializable64BitsSignature;
use ledgera_types::app_template::template::LedgeraApplicationTemplate;
use ledgera_types::digest::LedgeraDigest;

pub fn exec_print_graph<
    LAT: LedgeraApplicationTemplate,
    CmpPrinter: LedgeraComputationItemsPrinter<LAT>,
>(
    filename_to_print_graph: &str,
    k: &LedgeraKnowledgeRepresentation<LAT>,
    c_monikers: &HashMap<String, SerdeSerializable64BitsSignature>,
    d_monikers: &HashMap<String, LedgeraDigest>,
) {
    let svg_path = format!("{}.svg", filename_to_print_graph);
    print_current_knowledge_as_graph_v3::<LAT, CmpPrinter>(&svg_path, k, c_monikers, d_monikers);
    open_svg(&svg_path);
}

/// Try to open an SVG file using the host system's default handler.
/// This is non-blocking.
///
/// Logs a warning if launching fails.
fn open_svg<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();

    let result = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(path)
            .spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(path).spawn()
    } else {
        // Linux / BSD / etc.
        Command::new("xdg-open").arg(path).spawn()
    };

    if let Err(err) = result {
        log::warn!("Failed to open SVG file '{}': {}", path.display(), err);
    }
}
