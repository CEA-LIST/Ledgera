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

use std::thread::JoinHandle;

use eframe::glow;
use egui::{pos2, vec2, Color32, CursorIcon, FontId, Rect, Sense, Stroke, Vec2};

use crate::layout::{Direction, Layout};
use crate::terminal::Terminal;

const FONT_SIZE: f32 = 14.0;
const BORDER: f32 = 2.0;
const DIVIDER_HIT: f32 = 4.0; // half-width of the drag hit area on each side of a boundary

struct Divider {
    rect: Rect,
    path: Vec<usize>, // child-index path from root to the containing Split node
    child_idx: usize, // divider sits between child_idx and child_idx+1
    direction: Direction,
    available: f32, // pixel extent along the split axis (for frac↔pixel conversion)
}

struct Panel {
    command: Vec<String>,
    terminal: Option<Terminal>,
    spawned_proc: Option<JoinHandle<()>>,
    spawn_error: Option<String>,
}

pub struct TuiPanelsApp {
    layout: Layout,
    panels: Vec<Panel>,
    focused: usize,
    char_size: Option<Vec2>,
}

impl TuiPanelsApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, layout: Layout) -> Self {
        let mut panels = Vec::new();
        collect_panels(&layout, &mut panels);
        TuiPanelsApp {
            layout,
            panels,
            focused: 0,
            char_size: None,
        }
    }
}

// == Tree traversal helpers ====================================================

fn collect_panels(layout: &Layout, out: &mut Vec<Panel>) {
    match layout {
        Layout::Leaf { command } => {
            out.push(Panel {
                command: command.clone(),
                terminal: None,
                spawned_proc: None,
                spawn_error: None,
            });
        }
        Layout::Split { children, .. } => {
            for (_, child) in children {
                collect_panels(child, out);
            }
        }
    }
}

/// Walk the layout tree, filling `panel_rects` (one entry per leaf, in DFS order)
/// and `dividers` (one entry per inter-child boundary in every Split node).
fn compute_rects(
    layout: &Layout,
    rect: Rect,
    leaf_idx: &mut usize,
    panel_rects: &mut Vec<Rect>,
    dividers: &mut Vec<Divider>,
    path: &mut Vec<usize>,
) {
    match layout {
        Layout::Leaf { .. } => {
            panel_rects.push(rect);
            *leaf_idx += 1;
        }
        Layout::Split {
            direction,
            children,
        } => {
            let available = match direction {
                Direction::Horizontal => rect.width(),
                Direction::Vertical => rect.height(),
            };

            // Compute each child's rect.
            let mut offset = 0.0f32;
            let child_rects: Vec<Rect> = children
                .iter()
                .map(|(frac, _)| {
                    let size = frac * available;
                    let r = match direction {
                        Direction::Horizontal => Rect::from_min_size(
                            pos2(rect.min.x + offset, rect.min.y),
                            vec2(size, rect.height()),
                        ),
                        Direction::Vertical => Rect::from_min_size(
                            pos2(rect.min.x, rect.min.y + offset),
                            vec2(rect.width(), size),
                        ),
                    };
                    offset += size;
                    r
                })
                .collect();

            // One divider between each consecutive pair of children.
            for i in 0..children.len() - 1 {
                let boundary = match direction {
                    Direction::Horizontal => child_rects[i + 1].min.x,
                    Direction::Vertical => child_rects[i + 1].min.y,
                };
                let drect = match direction {
                    Direction::Horizontal => Rect::from_x_y_ranges(
                        (boundary - DIVIDER_HIT)..=(boundary + DIVIDER_HIT),
                        rect.min.y..=rect.max.y,
                    ),
                    Direction::Vertical => Rect::from_x_y_ranges(
                        rect.min.x..=rect.max.x,
                        (boundary - DIVIDER_HIT)..=(boundary + DIVIDER_HIT),
                    ),
                };
                dividers.push(Divider {
                    rect: drect,
                    path: path.clone(),
                    child_idx: i,
                    direction: direction.clone(),
                    available,
                });
            }

            for (i, (_, child)) in children.iter().enumerate() {
                path.push(i);
                compute_rects(child, child_rects[i], leaf_idx, panel_rects, dividers, path);
                path.pop();
            }
        }
    }
}

/// Navigate to the Split node identified by `path` and adjust the fractions of
/// children[child_idx] and children[child_idx+1] by `delta_frac`.
fn apply_drag(layout: &mut Layout, path: &[usize], child_idx: usize, delta_frac: f32) {
    match layout {
        Layout::Leaf { .. } => {}
        Layout::Split { children, .. } => {
            if path.is_empty() {
                let a = child_idx;
                let b = child_idx + 1;
                let total = children[a].0 + children[b].0;
                let new_a = (children[a].0 + delta_frac).clamp(0.05, total - 0.05);
                children[a].0 = new_a;
                children[b].0 = total - new_a;
            } else {
                apply_drag(&mut children[path[0]].1, &path[1..], child_idx, delta_frac);
            }
        }
    }
}

// == eframe::App ===============================================================

impl eframe::App for TuiPanelsApp {
    fn on_exit(&mut self, _gl: Option<&glow::Context>) {
        for panel in self.panels.drain(0..) {
            if let Some(mut term) = panel.terminal {
                term.kill();
            }
            if let Some(handle) = panel.spawned_proc {
                let _ = handle.join();
            }
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.char_size.is_none() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            let fid = FontId::monospace(FONT_SIZE);
            let w = ctx.fonts(|f| f.glyph_width(&fid, 'M'));
            let h = ctx.fonts(|f| f.row_height(&fid));
            self.char_size = Some(vec2(w, h));
        }
        let char_size = self.char_size.unwrap();

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::BLACK))
            .show(ctx, |ui| {
                let win = ui.available_rect_before_wrap();
                if win.width() <= 0.0 || win.height() <= 0.0 {
                    return;
                }

                // Recompute panel rects and dividers from the (possibly updated) tree.
                let mut panel_rects: Vec<Rect> = Vec::with_capacity(self.panels.len());
                let mut dividers: Vec<Divider> = Vec::new();
                let mut leaf_idx = 0usize;
                let mut path: Vec<usize> = Vec::new();
                compute_rects(
                    &self.layout,
                    win,
                    &mut leaf_idx,
                    &mut panel_rects,
                    &mut dividers,
                    &mut path,
                );

                // Focus on click — skip if the click lands on a divider.
                if ui.input(|i| i.pointer.primary_clicked()) {
                    if let Some(pos) = ctx.pointer_latest_pos() {
                        let on_divider = dividers.iter().any(|d| d.rect.contains(pos));
                        if !on_divider {
                            for (i, &r) in panel_rects.iter().enumerate() {
                                if r.contains(pos) {
                                    self.focused = i;
                                    break;
                                }
                            }
                        }
                    }
                }

                // Allocate divider interaction rects (must happen before painter clone).
                let div_responses: Vec<_> = dividers
                    .iter()
                    .map(|d| ui.allocate_rect(d.rect, Sense::drag()))
                    .collect();

                let painter = ui.painter().clone();

                // Process divider responses: set cursor and collect any drag delta.
                let mut pending_drag: Option<(Vec<usize>, usize, f32)> = None;
                for (div, resp) in dividers.iter().zip(div_responses.iter()) {
                    let cursor = match div.direction {
                        Direction::Horizontal => CursorIcon::ResizeHorizontal,
                        Direction::Vertical => CursorIcon::ResizeVertical,
                    };
                    if resp.hovered() || resp.dragged() {
                        ctx.set_cursor_icon(cursor);
                    }
                    if resp.dragged() {
                        let delta = resp.drag_delta();
                        let delta_frac = match div.direction {
                            Direction::Horizontal => delta.x / div.available,
                            Direction::Vertical => delta.y / div.available,
                        };
                        pending_drag = Some((div.path.clone(), div.child_idx, delta_frac));
                    }
                }
                if let Some((path, child_idx, delta_frac)) = pending_drag {
                    apply_drag(&mut self.layout, &path, child_idx, delta_frac);
                }

                // Collect keyboard events once, forward to the focused terminal.
                let events: Vec<egui::Event> = ctx.input(|i| i.events.clone());
                let focused_idx = self.focused;

                for (i, panel) in self.panels.iter_mut().enumerate() {
                    let Some(&panel_rect) = panel_rects.get(i) else {
                        continue;
                    };
                    let term_rect = panel_rect.shrink(BORDER);

                    let cols = ((term_rect.width() / char_size.x).floor() as u16).max(10);
                    let rows = ((term_rect.height() / char_size.y).floor() as u16).max(5);

                    if panel.spawned_proc.is_none() {
                        match Terminal::spawn(&panel.command, cols, rows) {
                            Ok((term, handle)) => {
                                panel.terminal = Some(term);
                                panel.spawned_proc = Some(handle);
                            }
                            Err(e) => panel.spawn_error = Some(e.to_string()),
                        }
                    }

                    if let Some(err) = &panel.spawn_error {
                        render_error(&painter, term_rect, &format!("spawn error:\n{}", err));
                    } else if let Some(term) = panel.terminal.as_mut() {
                        term.resize(cols, rows);

                        if i == focused_idx {
                            for ev in &events {
                                match ev {
                                    egui::Event::Text(text) => {
                                        term.write_input(text.as_bytes());
                                    }
                                    egui::Event::Key {
                                        key,
                                        pressed: true,
                                        modifiers,
                                        ..
                                    } => {
                                        if let Some(bytes) = key_to_bytes(*key, *modifiers) {
                                            term.write_input(&bytes);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        render_terminal(&painter, term, term_rect, char_size, i == focused_idx);
                    }

                    let border_color = if i == focused_idx {
                        Color32::from_rgb(0, 120, 255)
                    } else {
                        Color32::from_rgb(55, 55, 55)
                    };
                    painter.rect_stroke(panel_rect, 0.0, Stroke::new(BORDER, border_color));
                }

                ctx.request_repaint();
            });
    }
}

// == Rendering =================================================================

fn render_error(painter: &egui::Painter, rect: Rect, message: &str) {
    painter.rect_filled(rect, 0.0, Color32::from_rgb(80, 0, 0));
    painter.text(
        rect.min + vec2(6.0, 6.0),
        egui::Align2::LEFT_TOP,
        message,
        FontId::monospace(FONT_SIZE),
        Color32::from_rgb(255, 100, 100),
    );
}

fn render_terminal(
    painter: &egui::Painter,
    term: &Terminal,
    rect: Rect,
    char_size: Vec2,
    focused: bool,
) {
    painter.rect_filled(rect, 0.0, Color32::BLACK);

    let font_id = FontId::monospace(FONT_SIZE);
    let guard = term.parser.lock().unwrap();
    let screen = guard.screen();
    let (rows, cols) = screen.size();

    for row in 0..rows {
        for col in 0..cols {
            let Some(cell) = screen.cell(row, col) else {
                continue;
            };

            let x = rect.min.x + col as f32 * char_size.x;
            let y = rect.min.y + row as f32 * char_size.y;
            if x + char_size.x > rect.max.x + 0.5 || y + char_size.y > rect.max.y + 0.5 {
                continue;
            }

            let cell_rect = Rect::from_min_size(pos2(x, y), char_size);

            let bg = vt100_color(cell.bgcolor(), false);
            if bg != Color32::BLACK {
                painter.rect_filled(cell_rect, 0.0, bg);
            }

            let ch = cell.contents();
            if !ch.is_empty() && ch != " " {
                let fg = vt100_color(cell.fgcolor(), true);
                painter.text(pos2(x, y), egui::Align2::LEFT_TOP, ch, font_id.clone(), fg);
            }
        }
    }

    if focused {
        let (cr, cc) = screen.cursor_position();
        let cx = rect.min.x + cc as f32 * char_size.x;
        let cy = rect.min.y + cr as f32 * char_size.y;
        if cx + char_size.x <= rect.max.x && cy + char_size.y <= rect.max.y {
            painter.rect_filled(
                Rect::from_min_size(pos2(cx, cy), char_size),
                0.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 120),
            );
        }
    }
}

// == Color helpers =============================================================

fn vt100_color(color: vt100::Color, is_fg: bool) -> Color32 {
    match color {
        vt100::Color::Default => {
            if is_fg {
                Color32::from_gray(204)
            } else {
                Color32::BLACK
            }
        }
        vt100::Color::Idx(i) => ansi256(i),
        vt100::Color::Rgb(r, g, b) => Color32::from_rgb(r, g, b),
    }
}

fn ansi256(idx: u8) -> Color32 {
    match idx {
        0 => Color32::from_rgb(0, 0, 0),
        1 => Color32::from_rgb(170, 0, 0),
        2 => Color32::from_rgb(0, 170, 0),
        3 => Color32::from_rgb(170, 170, 0),
        4 => Color32::from_rgb(0, 0, 170),
        5 => Color32::from_rgb(170, 0, 170),
        6 => Color32::from_rgb(0, 170, 170),
        7 => Color32::from_rgb(170, 170, 170),
        8 => Color32::from_rgb(85, 85, 85),
        9 => Color32::from_rgb(255, 85, 85),
        10 => Color32::from_rgb(85, 255, 85),
        11 => Color32::from_rgb(255, 255, 85),
        12 => Color32::from_rgb(85, 85, 255),
        13 => Color32::from_rgb(255, 85, 255),
        14 => Color32::from_rgb(85, 255, 255),
        15 => Color32::from_rgb(255, 255, 255),
        16..=231 => {
            let v = idx - 16;
            let b = v % 6;
            let g = (v / 6) % 6;
            let r = v / 36;
            let f = |c: u8| if c == 0 { 0u8 } else { 55 + c * 40 };
            Color32::from_rgb(f(r), f(g), f(b))
        }
        232..=255 => {
            let l = 8u8.saturating_add((idx - 232).saturating_mul(10));
            Color32::from_gray(l)
        }
    }
}

// == Key → byte sequence =======================================================

fn key_to_bytes(key: egui::Key, mods: egui::Modifiers) -> Option<Vec<u8>> {
    if mods.ctrl && !mods.alt {
        let byte: u8 = match key {
            egui::Key::A => 0x01,
            egui::Key::B => 0x02,
            egui::Key::C => 0x03,
            egui::Key::D => 0x04,
            egui::Key::E => 0x05,
            egui::Key::F => 0x06,
            egui::Key::G => 0x07,
            egui::Key::H => 0x08,
            egui::Key::I => 0x09,
            egui::Key::J => 0x0a,
            egui::Key::K => 0x0b,
            egui::Key::L => 0x0c,
            egui::Key::M => 0x0d,
            egui::Key::N => 0x0e,
            egui::Key::O => 0x0f,
            egui::Key::P => 0x10,
            egui::Key::Q => 0x11,
            egui::Key::R => 0x12,
            egui::Key::S => 0x13,
            egui::Key::T => 0x14,
            egui::Key::U => 0x15,
            egui::Key::V => 0x16,
            egui::Key::W => 0x17,
            egui::Key::X => 0x18,
            egui::Key::Y => 0x19,
            egui::Key::Z => 0x1a,
            _ => return None,
        };
        return Some(vec![byte]);
    }

    Some(match key {
        egui::Key::Enter => vec![b'\r'],
        egui::Key::Backspace => vec![0x7f],
        egui::Key::Tab => vec![b'\t'],
        egui::Key::Escape => vec![0x1b],
        egui::Key::ArrowUp => vec![0x1b, b'[', b'A'],
        egui::Key::ArrowDown => vec![0x1b, b'[', b'B'],
        egui::Key::ArrowRight => vec![0x1b, b'[', b'C'],
        egui::Key::ArrowLeft => vec![0x1b, b'[', b'D'],
        egui::Key::Home => vec![0x1b, b'[', b'H'],
        egui::Key::End => vec![0x1b, b'[', b'F'],
        egui::Key::PageUp => vec![0x1b, b'[', b'5', b'~'],
        egui::Key::PageDown => vec![0x1b, b'[', b'6', b'~'],
        egui::Key::Delete => vec![0x1b, b'[', b'3', b'~'],
        egui::Key::Insert => vec![0x1b, b'[', b'2', b'~'],
        egui::Key::F1 => vec![0x1b, b'O', b'P'],
        egui::Key::F2 => vec![0x1b, b'O', b'Q'],
        egui::Key::F3 => vec![0x1b, b'O', b'R'],
        egui::Key::F4 => vec![0x1b, b'O', b'S'],
        _ => return None,
    })
}
