# Ledgera

Please refer to [the dedicated website for Ledgera](https://ledgera.tech).

## Ledgera White Paper

A high-level introduction to Ledgera (vision, design rationale, and target use cases) is given [here](https://ledgera.tech/assets/docs/ledgera_whitepaper_paper.pdf).

## Ledgera Yellow Paper

The current protocol-level specification, with formal definitions, data structures, and
the reference execution model is described
[here](https://ledgera.tech/assets/docs/ledgera_v_0_2_yellow_paper_final.pdf).

## Ledgera Tutorial

Find [here](https://ledgera.tech/assets/docs/ledgera_tutorial_v_1.pdf) a hands-on guide to
building a domain-specific application on the Ledgera BFT system, with core Ledgera and
Rust concepts, two worked reference apps (a minimal hello-world and a full atomic
register), and how to turn the blank template into your own application.

## Installation

> **Do not install Rust or cargo via your system package manager** (`apt install cargo`, `brew install rust`, etc.). Those versions are too old to read the lock file in this repo. Use rustup instead — it installs `rustc`, `cargo`, and the correct version automatically.

### 1. Install system prerequisites

- **Ubuntu / Debian**:
  ```sh
  sudo apt-get install build-essential pkg-config libssl-dev git python3
  ```
- **Fedora / RHEL**:
  ```sh
  sudo dnf install @development-tools pkgconf-pkg-config openssl-devel git curl python3
  ```
- **macOS**: `xcode-select --install`, then install Python 3 via [python.org](https://www.python.org/downloads/) or `brew install python`.
- **Windows**: install [Git for Windows](https://git-scm.com/download/win) and [Python 3](https://www.python.org/downloads/windows/). The MSVC C++ Build Tools are handled by rustup in step 2.

### 2. Install rustup

- **Linux / macOS**:
  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Windows**: download and run [rustup-init.exe](https://rustup.rs). 

After the installer completes, **open a new terminal** so that `cargo` and `rustc` are on your PATH.

The workspace pins Rust 1.89 via [code/rust-toolchain.toml](code/rust-toolchain.toml). Rustup will download and switch to that version automatically on the first `cargo` invocation inside `code/`.

### 3. Build

```sh
cd code
cargo build --release
```

### 4. Run a test

Each `crates/test_*` (non-TUI) crate ships a `launch_test.py` that builds a local multi-node testnet and spawns one terminal per node. From inside the crate directory:

```sh
python3 launch_test.py
```
