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
use syn::parse::{Parse, ParseStream};
use syn::{parse2, Ident, ItemEnum, LitStr, Token};

struct ServiceTopicsArgs {
    name: LitStr,
}

impl Parse for ServiceTopicsArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        if key != "name" {
            return Err(syn::Error::new(key.span(), "expected `name`"));
        }
        input.parse::<Token![=]>()?;
        let name: LitStr = input.parse()?;
        Ok(ServiceTopicsArgs { name })
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap());
    }
    result
}

pub fn impl_ledgera_service_topics(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args: ServiceTopicsArgs = match parse2(attr) {
        Ok(args) => args,
        Err(err) => return err.to_compile_error(),
    };
    let item: ItemEnum = match parse2(input) {
        Ok(item) => item,
        Err(err) => return err.to_compile_error(),
    };

    let name = &item.ident;
    let vis = &item.vis;
    let variants = &item.variants;
    let attrs = &item.attrs;
    let service_name = args.name.value();

    let match_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let variant_name = &v.ident;
            let variant_str = variant_name.to_string();
            let topic_snake = to_snake_case(&variant_str);

            // Variants whose name contains "Private" get the client name appended.
            if variant_str.contains("Private") {
                let fmt_str = format!("{}/{}/{{}}", service_name, topic_snake);
                quote! {
                    #name::#variant_name => format!(#fmt_str, name_as_client)
                }
            } else {
                let topic_str = format!("{}/{}", service_name, topic_snake);
                quote! {
                    #name::#variant_name => #topic_str.to_string()
                }
            }
        })
        .collect();

    quote! {
        #(#attrs)*
        #vis enum #name {
            #variants
        }

        impl #name {
            pub fn get_topic_str(&self, name_as_client: &str) -> String {
                match self {
                    #(#match_arms,)*
                }
            }
        }
    }
}
