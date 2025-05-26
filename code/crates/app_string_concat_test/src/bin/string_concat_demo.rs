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

use ledgera_util_basic_demo::launch::launch_demo;

fn target_path(x: &str) -> String {
    if cfg!(target_os = "windows") {
        panic!("windows not supported for the demo")
        //format!(r"..\..\target\release\{}.exe", x)
    } else {
        format!("../../target/release/{}", x)
    }
}

fn main() {
    launch_demo(
        "testnet",
        &target_path("testnet_maker"),
        &target_path("configurable_node_runner"),
        "cv",
        "sv",
        "csv",
        "l",
    )
}
