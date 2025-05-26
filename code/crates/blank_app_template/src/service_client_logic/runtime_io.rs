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

use ledgera_node_client::comms::feedback_from_core_client::ValidatedCoreFeedbackMessage;

use crate::lat_binding::LedgeraServiceTemplate;
use crate::service_client_logic::HighLevelServiceUserRequests;

pub struct ServiceClientRuntimeIO {
    // a sender to send requests to your service client (e.g., from a user interface CLI/TUI/GUI)
    pub user_requests_sender: tokio::sync::mpsc::Sender<HighLevelServiceUserRequests>,
    // a receiver so that your service client might receive information from the Ledgera Core client it is co-located with
    pub validated_core_msgs_receiver:
        tokio::sync::mpsc::Receiver<ValidatedCoreFeedbackMessage<LedgeraServiceTemplate>>,
}
