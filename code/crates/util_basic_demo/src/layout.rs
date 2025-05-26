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

use crate::app::TuiPanelsApp;

#[derive(Debug, Clone)]
pub enum Direction {
    Horizontal,
    Vertical,
}

/// A layout node: either a leaf (a command to run) or a split.
///
/// Fractions in a Split must sum to 1.0.  They describe the proportion of the
/// available width (Horizontal) or height (Vertical) allocated to each child.
#[derive(Debug, Clone)]
pub enum Layout {
    Leaf {
        command: Vec<String>,
    },
    Split {
        direction: Direction,
        children: Vec<(f32, Layout)>,
    },
}

pub fn run(layout: Layout, title: &str) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(title)
            .with_maximized(true),
        ..Default::default()
    };
    eframe::run_native(
        title,
        options,
        Box::new(|cc| Ok(Box::new(TuiPanelsApp::new(cc, layout)))),
    )
}
