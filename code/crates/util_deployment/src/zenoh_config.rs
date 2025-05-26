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

pub const PLACEHOLDER_NAME: &str = "PLACEHOLDER_NAME";

pub const RAW_ZENOH_CONFIG: &str = r#"
{
  mode: "peer",
  metadata: {
    name: "PLACEHOLDER_NAME",
    location: "location",
  },
  connect: {
    timeout_ms: { router: -1, peer: -1, client: 0 },
    endpoints: [],
    exit_on_failure: { router: false, peer: false, client: true },
    retry: {
      period_init_ms: 1000,
      period_max_ms: 4000,
      period_increase_factor: 2,
    },
  },
  listen: {
    timeout_ms: 0,
    endpoints: { router: ["tcp/[::]:7447"], peer: ["tcp/0.0.0.0:0"] },
    exit_on_failure: true,
    retry: {
      period_init_ms: 1000,
      period_max_ms: 4000,
      period_increase_factor: 2,
    },
  },
  open: {
    return_conditions: {
      connect_scouted: true,
      declares: true,
    },
  },
  scouting: {
    timeout: 3000,
    delay: 500,
    multicast: {
      enabled: true,
      address: "224.0.0.224:7446",
      interface: "auto",
      ttl: 1,
      autoconnect: { router: [], peer: ["router", "peer"] },
      listen: true,
    },
    gossip: {
      enabled: true,
      multihop: false,
      autoconnect: { router: [], peer: ["router", "peer"] },
    },
  },
  timestamping: {
    enabled: { router: true, peer: false, client: false },
    drop_future_timestamp: false,
  },
  queries_default_timeout: 10000,
  transport: {
    unicast: {
      accept_timeout: 10000,
      accept_pending: 100,
      max_sessions: 1000,
      max_links: 1,
      lowlatency: false,
      qos: {
        enabled: true,
      },
      compression: {
        enabled: false,
      },
    },
    multicast: {
      join_interval: 2500,
      max_sessions: 1000,
      qos: {
        enabled: false,
      },
      compression: {
        enabled: false,
      },
    },
    link: {
      tx: {
        sequence_number_resolution: "32bit",
        lease: 10000,
        keep_alive: 4,
        batch_size: 65535,
        queue: {
          size: {
            control: 1,
            real_time: 1,
            interactive_high: 1,
            interactive_low: 1,
            data_high: 2,
            data: 4,
            data_low: 4,
            background: 4,
          },
          congestion_control: {
            drop: {
              wait_before_drop: 1000,
            },
            block: {
              wait_before_close: 5000000,
            },
          },
          batching: {
            enabled: true,
            time_limit: 1,
          },
        },
      },
      rx: {
        buffer_size: 65535,
        max_message_size: 1073741824,
      },
      tls: {
        root_ca_certificate: null,
        listen_private_key: null,
        listen_certificate: null,
        enable_mtls: false,
        connect_private_key: null,
        connect_certificate: null,
        verify_name_on_connect: true,
      },
    },
    shared_memory: {
      enabled: true,
    },
    auth: {
      usrpwd: {
        user: null,
        password: null,
        dictionary_file: null,
      },
      pubkey: {
        public_key_pem: null,
        private_key_pem: null,
        public_key_file: null,
        private_key_file: null,
        key_size: null,
        known_keys_file: null,
      },
    },
  },
  adminspace: {
    enabled: false,
    permissions: {
      read: true,
      write: false,
    },
  },
}
"#;
