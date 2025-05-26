/**************************************************************************************************
 * Copyright (c) 2025 CEA (Commissariat à l'énergie atomique et aux énergies alternatives)
 *   contributors:
 *   - Ryane Djari ( ryane.djari@cea.fr )
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

mod computation;
mod data;
mod error;
mod predicate;
mod service;
mod service_msg;
mod service_topics;
mod tag;

use proc_macro::TokenStream;

/// Adds standard derives and a `LedgeraCommunicatableItem` impl to a data enum.
#[proc_macro_attribute]
pub fn ledgera_data(_attr: TokenStream, input: TokenStream) -> TokenStream {
    data::impl_ledgera_data(input.into()).into()
}

/// Adds standard derives and a `LedgeraCommunicatableItem` impl to an error enum.
#[proc_macro_attribute]
pub fn ledgera_error(_attr: TokenStream, input: TokenStream) -> TokenStream {
    error::impl_ledgera_error(input.into()).into()
}

/// Adds standard derives and a `LedgeraCommunicatableItem` impl to an operation enum.
#[proc_macro_attribute]
pub fn ledgera_computation(_attr: TokenStream, input: TokenStream) -> TokenStream {
    computation::impl_ledgera_computation(input.into()).into()
}

/// Adds standard derives, `LedgeraCommunicatableItem`, and `LedgeraAtomicTag` impls to a tag enum.
#[proc_macro_attribute]
pub fn ledgera_tag(_attr: TokenStream, input: TokenStream) -> TokenStream {
    tag::impl_ledgera_tag(input.into()).into()
}

/// Adds standard derives and a `LedgeraCommunicatableItem` impl to a singular-argument predicate enum.
#[proc_macro_attribute]
pub fn ledgera_local_predicate(_attr: TokenStream, input: TokenStream) -> TokenStream {
    predicate::impl_ledgera_local_predicate(input.into()).into()
}

/// Adds standard derives and a `LedgeraCommunicatableItem` impl to a multi-argument (global)
/// predicate struct or enum.
#[proc_macro_attribute]
pub fn ledgera_global_predicate(_attr: TokenStream, input: TokenStream) -> TokenStream {
    predicate::impl_ledgera_global_predicate(input.into()).into()
}

/// Generates the `LedgeraApplicationTemplate` impl for the annotated service struct, plus
/// delegate impls of `LedgeraAtomicComputation`, `LedgeraOperationSingularArgumentPredicate`, and
/// `LedgeraOperationMultiArgumentsPredicate` that forward to user-written inherent methods named
/// `compute` / `is_valid_for` on the operation, predicate, and global-predicate types.
///
/// Required arguments: `name`, `data`, `operation`, `tag`, `local_predicate`, `global_predicate`, `error`.
#[proc_macro_attribute]
pub fn ledgera_application_template(attr: TokenStream, input: TokenStream) -> TokenStream {
    service::impl_ledgera_application_template(attr.into(), input.into()).into()
}

/// Adds standard derives and a `LedgeraPublishableMessage` impl to a service-message struct.
///
/// Required argument: `msg_type = "..."`.
#[proc_macro_attribute]
pub fn ledgera_service_msg(attr: TokenStream, input: TokenStream) -> TokenStream {
    service_msg::impl_ledgera_service_msg(attr.into(), input.into()).into()
}

/// Generates `get_topic_str(&self, name_as_client: &str) -> String` for a service-topic enum.
/// Variants whose name contains "Private" get the client name appended to the topic string.
///
/// Required argument: `name = "..."`.
#[proc_macro_attribute]
pub fn ledgera_service_topics(attr: TokenStream, input: TokenStream) -> TokenStream {
    service_topics::impl_ledgera_service_topics(attr.into(), input.into()).into()
}
