/**************************************************************************************************
 * Copyright (c) 2025 CEA (Commissariat à l'énergie atomique et aux énergies alternatives)
 *   contributors:
 *   - Ryane Djari ( ryane.djari@cea.fr )
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

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemEnum};

pub fn impl_ledgera_tag(input: TokenStream) -> TokenStream {
    let item: ItemEnum = match parse2(input) {
        Ok(item) => item,
        Err(err) => return err.to_compile_error(),
    };
    let name = &item.ident;
    let vis = &item.vis;
    let variants = &item.variants;
    let attrs = &item.attrs;
    let generics = &item.generics;

    quote! {
        #(#attrs)*
        #[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
        #vis enum #name #generics {
            #variants
        }

        impl ledgera_types::traits::LedgeraCommunicatableItem for #name {}
        impl ledgera_types::app_template::operation::LedgeraAtomicTag for #name {}
    }
}
