

# Ledgera prototype

Please refer to [the dedicated website for Ledgera version V_0_1](https://docs.ledgera.tech/docs/versions/v_0_1/).


## Protocol specification

A high-level description of the protocol is given [here](https://docs.ledgera.tech/docs/versions/v_0_1/2_spec/).



## Software architecture

The current software architecture of the code is described [here](https://docs.ledgera.tech/docs/versions/v_0_1/3_implem/#architecture).


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
cargo build
```

### 4. Run a test

Each `crates/test_*` (non-TUI) crate ships a `launch_test.py` that builds a local multi-node testnet and spawns one terminal per node. From inside the crate directory:

```sh
python3 launch_test.py
```
