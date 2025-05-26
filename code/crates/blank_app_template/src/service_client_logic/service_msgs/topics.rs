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

use ledgera_macros::ledgera_service_topics;

/// Pub/sub topics this service publishes and subscribes to.
#[ledgera_service_topics(name = "template")]
pub enum ServicesTemplateDedicatedTopics {
    // TODO : for any channel of communication you want to define
    // so that your Service client may communicate outside of Ledgera Core
    // you must define a topic here
    DefaultTopic,
    /// Variants whose name contains "Private" are unique per client.
    PrivateTopic,
}
