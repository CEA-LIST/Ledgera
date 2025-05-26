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

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Item};

pub fn impl_ledgera_data(input: TokenStream) -> TokenStream {
    let item: Item = match parse2(input) {
        Ok(item) => item,
        Err(err) => return err.to_compile_error(),
    };

    match item {
        Item::Enum(item) => {
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
            }
        }
        Item::Struct(item) => {
            let name = &item.ident;
            let vis = &item.vis;
            let fields = &item.fields;
            let attrs = &item.attrs;
            let generics = &item.generics;

            quote! {
                #(#attrs)*
                #[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
                #vis struct #name #generics #fields

                impl ledgera_types::traits::LedgeraCommunicatableItem for #name {}
            }
        }
        _ => syn::Error::new_spanned(item, "ledgera_data can only be applied to structs or enums")
            .to_compile_error(),
    }
}
