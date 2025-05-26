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

use ledgera_pki::error::LedgeraPkiError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LedgeraCommunicationErrorContext {
    AtStartup,
    WhenQuerying(&'static str),
    WhenPublishing(&'static str),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LedgeraCommunicationError<SessError> {
    SessionError(SessError),
    PkiError(LedgeraPkiError),
    InContext(LedgeraCommunicationErrorContext, Box<Self>),
}
