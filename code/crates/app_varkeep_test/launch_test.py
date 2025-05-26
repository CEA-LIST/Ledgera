#**************************************************************************************************
# * Copyright (c) 2025 CEA (Commissariat à l'énergie atomique et aux énergies alternatives)
# *
# * This program and the accompanying materials are made
# * available under the terms of the Eclipse Public License 2.0
# * which is available at https://www.eclipse.org/legal/epl-2.0/
# *
# * SPDX-License-Identifier: Apache-2.0  
# *************************************************************************************************

import os
import platform



def init_windows():
    WIN_CMD = '..\\..\\target\\release\\service_varkeep_testnet_maker.exe 4 3'
    os.system(WIN_CMD)


def run_windows():
    WIN_COMMANDS = [
        '..\\..\\target\\release\\service_varkeep_node_runner.exe node1 c service_varkeep_testnet\\node1',
        '..\\..\\target\\release\\service_varkeep_node_runner.exe node2 cv service_varkeep_testnet\\node2',
        '..\\..\\target\\release\\service_varkeep_node_runner.exe node3 csv service_varkeep_testnet\\node3',
        '..\\..\\target\\release\\service_varkeep_node_runner.exe node4 vl service_varkeep_testnet\\node4'
    ]
    for cmd in WIN_COMMANDS:
        os.system(f'start cmd /k "{cmd}"')
    

def init_linux():
    UNX_CMD = '../../target/release/service_varkeep_testnet_maker 4 3'
    os.system(UNX_CMD)

def run_linux():
    UNX_COMMANDS = [
        '../../target/release/service_varkeep_node_runner node1 cv ./service_varkeep_testnet/node1',
        '../../target/release/service_varkeep_node_runner node2 csv ./service_varkeep_testnet/node2',
        '../../target/release/service_varkeep_node_runner node3 cs ./service_varkeep_testnet/node3',
        '../../target/release/service_varkeep_node_runner node4 vl ./service_varkeep_testnet/node4'
    ]
    for cmd in UNX_COMMANDS:
        os.system(f'gnome-terminal -- bash -c "{cmd}; exec bash"')



def check_bin_build_windows():
    if not os.path.isfile('..\\..\\target\\release\\service_varkeep_node_runner.exe'):
        os.system("cargo build --release") 



def check_bin_build_linux():
    if not os.path.isfile('../../target/release/service_varkeep_node_runner'):
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
    else:
        print(f"Unsupported OS: {system}")
