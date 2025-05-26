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


    

def init_linux():
    UNX_CMD = '../../target/release/string_concat_demo'
    os.system(UNX_CMD)



def check_bin_build_linux():
    if not os.path.isfile('../../target/release/configurable_node_runner'):
        os.system("cargo build --release") 


if __name__ == '__main__':
    system = platform.system()
    if system == "Windows":
        print(f"Unsupported OS: {system}")
    elif system == "Linux":
        check_bin_build_linux()
        init_linux()
    else:
        print(f"Unsupported OS: {system}")
