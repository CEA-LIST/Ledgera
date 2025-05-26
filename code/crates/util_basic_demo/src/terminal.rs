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

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub struct Terminal {
    pub parser: Arc<Mutex<vt100::Parser>>,
    writer: Box<dyn Write + Send>,
    platform: Platform,
    pub cols: u16,
    pub rows: u16,
}

// == Platform-specific internals ===============================================

/// Unix: full PTY via portable-pty.
#[cfg(not(target_os = "windows"))]
struct Platform {
    master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    _child: Box<dyn portable_pty::Child + Send + Sync>,
}

/// Windows: plain piped I/O via std::process::Command.
///
/// ConPTY (the portable-pty Windows backend) has known reliability issues with
/// try_clone_reader() — the output pipe frequently delivers no data.  Direct
/// pipes are simpler and work correctly: killing the child closes its write
/// ends, which gives the reader threads EOF and lets join() return cleanly.
#[cfg(target_os = "windows")]
struct Platform {
    child: std::process::Child,
}

// == Public API ================================================================

impl Terminal {
    pub fn spawn(
        command: &[String],
        cols: u16,
        rows: u16,
    ) -> Result<(Self, JoinHandle<()>), Box<dyn std::error::Error>> {
        #[cfg(not(target_os = "windows"))]
        return Self::spawn_pty(command, cols, rows);

        #[cfg(target_os = "windows")]
        return Self::spawn_pipes(command, cols, rows);
    }

    /// Kill the child process.
    ///
    /// On Unix this also drops the PTY master, which closes the master fd and
    /// sends HUP — that unblocks the reader thread's read() call.
    ///
    /// On Windows killing the child closes its stdout/stderr write ends, which
    /// gives the reader threads EOF and lets them exit.
    pub fn kill(&mut self) {
        #[cfg(not(target_os = "windows"))]
        {
            let _ = self.platform._child.kill();
            self.platform.master = None;
        }
        #[cfg(target_os = "windows")]
        {
            let _ = self.platform.child.kill();
        }
    }

    pub fn write_input(&mut self, data: &[u8]) {
        let _ = self.writer.write_all(data);
        let _ = self.writer.flush();
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        if self.cols == cols && self.rows == rows {
            return;
        }
        self.cols = cols;
        self.rows = rows;

        #[cfg(not(target_os = "windows"))]
        {
            // Signal the child to redraw at the new size (SIGWINCH).
            if let Some(m) = &self.platform.master {
                use portable_pty::PtySize;
                let _ = m.resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                });
            }
            // Resize the parser in-place instead of replacing it with a blank one.
            // TUI children (ratatui) will overwrite with a full redraw after SIGWINCH;
            // stdout-only children never redraw, so their existing content must be kept.
            self.parser
                .lock()
                .unwrap()
                .screen_mut()
                .set_size(rows, cols);
        }

        // On Windows, plain pipes carry no resize signal.  Resetting the parser
        // would blank the screen permanently because ratatui (and similar TUIs)
        // won't redraw unless they receive a resize event.  Instead, keep the
        // existing screen state — ratatui's own draw loop continuously outputs
        // frames, so the parser stays populated at whatever size the child chose.
    }
}

// == Unix spawn ================================================================

#[cfg(not(target_os = "windows"))]
impl Terminal {
    fn spawn_pty(
        command: &[String],
        cols: u16,
        rows: u16,
    ) -> Result<(Self, JoinHandle<()>), Box<dyn std::error::Error>> {
        use portable_pty::{native_pty_system, CommandBuilder, PtySize};

        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new(command[0].as_str());
        for arg in &command[1..] {
            cmd.arg(arg.as_str());
        }
        cmd.env("TERM", "xterm-256color");
        if let Ok(cwd) = std::env::current_dir() {
            cmd.cwd(cwd);
        }

        let master = pair.master;
        let child = pair.slave.spawn_command(cmd)?;

        let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 0)));
        let parser_clone = parser.clone();

        let mut reader = master.try_clone_reader()?;
        let handle = std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        parser_clone.lock().unwrap().process(&buf[..n]);
                    }
                }
            }
        });

        let writer: Box<dyn Write + Send> = master.take_writer()?;

        Ok((
            Terminal {
                parser,
                writer,
                platform: Platform {
                    master: Some(master),
                    _child: child,
                },
                cols,
                rows,
            },
            handle,
        ))
    }
}

// == Windows spawn =============================================================

#[cfg(target_os = "windows")]
impl Terminal {
    fn spawn_pipes(
        command: &[String],
        cols: u16,
        rows: u16,
    ) -> Result<(Self, JoinHandle<()>), Box<dyn std::error::Error>> {
        use std::process::{Command, Stdio};

        let mut child = Command::new(&command[0])
            .args(&command[1..])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()?;

        let parser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 0)));

        let stdout = child.stdout.take().ok_or("child stdout unavailable")?;
        let stderr = child.stderr.take().ok_or("child stderr unavailable")?;
        let stdin = child.stdin.take().ok_or("child stdin unavailable")?;

        let pa = parser.clone();
        let pb = parser.clone();

        // One thread per stream; an outer thread joins both so the caller
        // gets a single JoinHandle that completes only when both are done.
        let t_out = std::thread::spawn(move || read_into_parser(stdout, pa));
        let t_err = std::thread::spawn(move || read_into_parser(stderr, pb));
        let handle = std::thread::spawn(move || {
            let _ = t_out.join();
            let _ = t_err.join();
        });

        let writer: Box<dyn Write + Send> = Box::new(stdin);

        Ok((
            Terminal {
                parser,
                writer,
                platform: Platform { child },
                cols,
                rows,
            },
            handle,
        ))
    }
}

#[cfg(target_os = "windows")]
fn read_into_parser(mut src: impl Read, parser: Arc<Mutex<vt100::Parser>>) {
    let mut buf = [0u8; 4096];
    loop {
        match src.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                parser.lock().unwrap().process(&buf[..n]);
            }
        }
    }
}
