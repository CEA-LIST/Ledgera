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
use syn::parse::{Parse, ParseStream};
use syn::{parse2, Ident, ItemStruct, LitStr, Token};

struct ServiceArgs {
    name: LitStr,
    data: Ident,
    computation: Ident,
    tag: Ident,
    local_predicate: Ident,
    global_predicate: Ident,
    error: Ident,
}

impl Parse for ServiceArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut data = None;
        let mut computation = None;
        let mut tag = None;
        let mut local_predicate = None;
        let mut global_predicate = None;
        let mut error = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            match key.to_string().as_str() {
                "name" => name = Some(input.parse::<LitStr>()?),
                "data" => data = Some(input.parse::<Ident>()?),
                "computation" => computation = Some(input.parse::<Ident>()?),
                "tag" => tag = Some(input.parse::<Ident>()?),
                "local_predicate" => local_predicate = Some(input.parse::<Ident>()?),
                "global_predicate" => global_predicate = Some(input.parse::<Ident>()?),
                "error" => error = Some(input.parse::<Ident>()?),
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown argument: `{}`. Expected one of: name, data, computation, tag, local_predicate, global_predicate, error", other),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(ServiceArgs {
            name: name.ok_or_else(|| input.error("missing `name` argument"))?,
            data: data.ok_or_else(|| input.error("missing `data` argument"))?,
            computation: computation
                .ok_or_else(|| input.error("missing `computation` argument"))?,
            tag: tag.ok_or_else(|| input.error("missing `tag` argument"))?,
            local_predicate: local_predicate
                .ok_or_else(|| input.error("missing `local_predicate` argument"))?,
            global_predicate: global_predicate
                .ok_or_else(|| input.error("missing `global_predicate` argument"))?,
            error: error.ok_or_else(|| input.error("missing `error` argument"))?,
        })
    }
}

pub fn impl_ledgera_application_template(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args: ServiceArgs = match parse2(attr) {
        Ok(args) => args,
        Err(err) => return err.to_compile_error(),
    };
    let item: ItemStruct = match parse2(input) {
        Ok(item) => item,
        Err(err) => return err.to_compile_error(),
    };

    let struct_name = &item.ident;
    let vis = &item.vis;
    let attrs = &item.attrs;
    let service_name = &args.name;
    let data_ty = &args.data;
    let op_ty = &args.computation;
    let tag_ty = &args.tag;
    let pred_ty = &args.local_predicate;
    let multi_args_pred_ty = &args.global_predicate;
    let err_ty = &args.error;

    quote! {
        #(#attrs)*
        #vis struct #struct_name;

        impl ledgera_types::app_template::template::LedgeraApplicationTemplate for #struct_name {
            type RuntimeError = #err_ty;
            type Data = #data_ty;
            type Computation = #op_ty;
            type Tag = #tag_ty;
            type LocalPredicate = #pred_ty;
            type GlobalPredicate = #multi_args_pred_ty;

            fn get_service_name(&self) -> &str {
                #service_name
            }
        }


        impl ledgera_types::app_template::operation::LedgeraAtomicComputation<
            #data_ty, #err_ty
        > for #op_ty {
            fn compute(
                &self,
                arguments: ::std::vec::Vec<#data_ty>,
            ) -> impl ::std::future::Future<Output = ::std::result::Result<#data_ty, #err_ty>> + Send {
                self.ledgera_compute(arguments)
            }
        }

        impl ledgera_types::app_template::predicates::LedgeraOperationSingularArgumentPredicate<
            #data_ty, #err_ty
        > for #pred_ty {
            fn is_valid_for(
                &self,
                value: &#data_ty,
                function_instance_identifier: &ledgera_pki::manager::SerdeSerializable64BitsSignature,
            ) -> ::std::result::Result<bool, #err_ty> {
                self.ledgera_single_arg_is_valid(value, function_instance_identifier)
            }
        }

        impl ledgera_types::app_template::predicates::LedgeraOperationMultiArgumentsPredicate<
            #data_ty, #err_ty
        > for #multi_args_pred_ty {
            fn is_valid_for(
                &self,
                arguments: &[&#data_ty],
            ) -> ::std::result::Result<bool, #err_ty> {
                self.ledgera_multi_args_is_valid(arguments)
            }
        }
    }
}
