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

use crate::layout::{Direction, Layout};
use std::process::Command;

pub fn launch_demo(
    testnet_folder_name: &str,
    testnet_maker_bin_path: &str,
    node_runner_bin_path: &str,
    node1_conf: &str,
    node2_conf: &str,
    node3_conf: &str,
    node4_conf: &str,
) {
    // Initialise the testnet
    Command::new(testnet_maker_bin_path)
        .arg("4")
        .status()
        .expect("failed to run testnet_maker");

    let node_cmd = |name: &str, conf: &str| -> Vec<String> {
        vec![
            node_runner_bin_path.to_string(),
            name.to_string(),
            conf.to_string(),
            format!("./{testnet_folder_name}/{name}"),
        ]
    };

    let layout = Layout::Split {
        direction: Direction::Horizontal,
        children: vec![
            (
                0.5,
                Layout::Leaf {
                    command: node_cmd("node1", node1_conf),
                },
            ),
            (
                0.5,
                Layout::Split {
                    direction: Direction::Vertical,
                    children: vec![
                        (
                            1.0 / 3.0,
                            Layout::Leaf {
                                command: node_cmd("node2", node2_conf),
                            },
                        ),
                        (
                            1.0 / 3.0,
                            Layout::Leaf {
                                command: node_cmd("node3", node3_conf),
                            },
                        ),
                        (
                            1.0 / 3.0,
                            Layout::Leaf {
                                command: node_cmd("node4", node4_conf),
                            },
                        ),
                    ],
                },
            ),
        ],
    };

    crate::layout::run(layout, "testnet").unwrap();
}
