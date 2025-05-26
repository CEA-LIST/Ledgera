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
use syn::{parse2, Ident, ItemStruct, LitStr, Token};

struct ServiceMsgArgs {
    msg_type: LitStr,
}

impl Parse for ServiceMsgArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        if key != "msg_type" {
            return Err(syn::Error::new(key.span(), "expected `msg_type`"));
        }
        input.parse::<Token![=]>()?;
        let msg_type: LitStr = input.parse()?;
        Ok(ServiceMsgArgs { msg_type })
    }
}

pub fn impl_ledgera_service_msg(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args: ServiceMsgArgs = match parse2(attr) {
        Ok(args) => args,
        Err(err) => return err.to_compile_error(),
    };
    let item: ItemStruct = match parse2(input) {
        Ok(item) => item,
        Err(err) => return err.to_compile_error(),
    };

    let name = &item.ident;
    let vis = &item.vis;
    let fields = &item.fields;
    let attrs = &item.attrs;
    let msg_type_str = &args.msg_type;

    quote! {
        #(#attrs)*
        #[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
        #vis struct #name #fields

        impl ledgera_types::traits::LedgeraPublishableMessage for #name {
            fn get_msg_type() -> &'static str {
                #msg_type_str
            }
        }
    }
}
