#**************************************************************************************************
# * Copyright (c) 2025 CEA (Commissariat à l'énergie atomique et aux énergies alternatives)
# *
# * Licensed under the Apache License, Version 2.0 (the "License");
# * you may not use this file except in compliance with the License.
# * You may obtain a copy of the License at
# *
# *       https://www.apache.org/licenses/LICENSE-2.0
# *
# * Unless required by applicable law or agreed to in writing, software
# * distributed under the License is distributed on an "AS IS" BASIS,
# * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# * See the License for the specific language governing permissions and
# * limitations under the License.
# *
# * SPDX-License-Identifier: Apache-2.0
# *************************************************************************************************

import os
import platform


# Topology of the test cluster. Edit this list to add / remove nodes or change roles.
# Each entry: (node_name, roles_abbrs)
#   roles_abbrs is any combination of:
#     'c' = client     (gets the TUI)
#     's' = persistent storage
#     'v' = voter / computer
#     'l' = secure logger
NODES = [
    ("node1", "cv"),
    ("node2", "sv"),
    ("node3", "cs"),
    ("node4", "vl"),
]

N_CLIENTS = sum(1 for (_, roles) in NODES if "c" in roles)

TESTNET_DIR_NAME = "service_template_testnet"
TESTNET_MAKER = "service_template_testnet_maker"
NODE_RUNNER = "service_template_node_runner"


def _bin_path(sep, exe_suffix, binary_name):
    return f'..{sep}..{sep}target{sep}release{sep}{binary_name}{exe_suffix}'


def _maker_cmd(sep, exe_suffix):
    return f'{_bin_path(sep, exe_suffix, TESTNET_MAKER)} {len(NODES)} {N_CLIENTS}'


def _runner_cmd(sep, exe_suffix, name, roles):
    pki = f'{TESTNET_DIR_NAME}{sep}{name}' if sep == "\\" else f'./{TESTNET_DIR_NAME}/{name}'
    return f'{_bin_path(sep, exe_suffix, NODE_RUNNER)} {name} {roles} {pki}'


def init_windows():
    os.system(_maker_cmd("\\", ".exe"))


def init_linux():
    os.system(_maker_cmd("/", ""))


def init_macos():
    os.system(_maker_cmd("/", ""))


def run_windows():
    for name, roles in NODES:
        cmd = _runner_cmd("\\", ".exe", name, roles)
        os.system(f'start cmd /k "{cmd}"')


def run_linux():
    for name, roles in NODES:
        cmd = _runner_cmd("/", "", name, roles)
        os.system(f'gnome-terminal -- bash -c "{cmd}; exec bash"')


def run_macos():
    for name, roles in NODES:
        cmd = _runner_cmd("/", "", name, roles)
        os.system(
            f"osascript -e 'tell app \"Terminal\" to do script \"cd {os.getcwd()} && {cmd}\"'"
        )


def check_bin_build_windows():
    if not os.path.isfile(_bin_path("\\", ".exe", NODE_RUNNER)):
        os.system("cargo build --release")


def check_bin_build_linux():
    if not os.path.isfile(_bin_path("/", "", NODE_RUNNER)):
        os.system("cargo build --release")


def check_bin_build_macos():
    if not os.path.isfile(_bin_path("/", "", NODE_RUNNER)):
        os.system("cargo build --release")


if __name__ == '__main__':
    system = platform.system()
    if system == "Windows":
        check_bin_build_windows()
        init_windows()
        run_windows()
    elif system == "Linux":
        check_bin_build_linux()
        init_linux()
        run_linux()
    elif system == "Darwin":
        check_bin_build_macos()
        init_macos()
        run_macos()
    else:
        print(f"Unsupported OS: {system}")
