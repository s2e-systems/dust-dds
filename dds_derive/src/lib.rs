mod attribute_helpers;
mod derive;

use derive::{
    cdr_deserialize::expand_cdr_deserialize, cdr_representation::expand_cdr_representation,
    cdr_serialize::expand_cdr_serialize, dds_borrow_key_holder::expand_dds_borrow_key_holder,
    dds_has_key::expand_has_key, dds_owning_key_holder::expand_dds_owning_key_holder,
    parameter_list_serialize::expand_parameter_list_serialize,
};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, FnArg, ItemImpl};

#[proc_macro_derive(DdsHasKey, attributes(key))]
pub fn derive_dds_has_key(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_has_key(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DdsBorrowKeyHolder, attributes(key))]
pub fn derive_dds_borrow_key_holder(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_dds_borrow_key_holder(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DdsOwningKeyHolder, attributes(key))]
pub fn derive_dds_owning_key_holder(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_dds_owning_key_holder(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(CdrRepresentation)]
pub fn derive_cdr_representation(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_cdr_representation(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(CdrSerialize)]
pub fn derive_cdr_serialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_cdr_serialize(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(CdrDeserialize)]
pub fn derive_cdr_deserialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_cdr_deserialize(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(ParameterListSerialize)]
pub fn derive_parameter_list_serialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_parameter_list_serialize(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DdsType, attributes(key))]
pub fn derive_dds_type(input: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();

    output.extend(derive_dds_has_key(input.clone()));
    output.extend(derive_dds_borrow_key_holder(input.clone()));
    output.extend(derive_dds_owning_key_holder(input.clone()));
    output.extend(derive_cdr_serialize(input.clone()));
    output.extend(derive_cdr_deserialize(input.clone()));
    output.extend(derive_cdr_representation(input));

    output
}

/// Attribute macro to generate the actor interface from
#[proc_macro_attribute]
pub fn actor_interface(
    _attribute_token_stream: TokenStream,
    item_token_stream: TokenStream,
) -> TokenStream {
    fn get_actor_interface_token_stream(input: &ItemImpl) -> proc_macro2::TokenStream {
        let actor_type = match input.self_ty.as_ref() {
            syn::Type::Path(p) => p,
            _ => panic!("Expect impl block with type"),
        };

        let mut actor_structs = proc_macro2::TokenStream::new();

        for method in input.items.iter().filter_map(|i| match i {
            syn::ImplItem::Fn(m) => Some(m),
            _ => None,
        }) {
            let method_ident = &method.sig.ident;
            assert!(
                method.sig.asyncness.is_some(),
                "Actor methods must be async"
            );

            let mut argument_ident_type_token_stream = proc_macro2::TokenStream::new();
            for argument in method.sig.inputs.iter().filter_map(|a| match a {
                FnArg::Receiver(_) => None,
                FnArg::Typed(t) => Some(t),
            }) {
                argument_ident_type_token_stream.extend(quote! { #argument, })
            }

            // To allow marking the generated code as for example #[allow(clippy:too_many_arguments)]
            let mut method_attributes_token_stream = proc_macro2::TokenStream::new();
            for attribute in &method.attrs {
                method_attributes_token_stream.extend(attribute.to_token_stream());
            }

            let mut argument_ident_token_stream = proc_macro2::TokenStream::new();
            for argument_ident in method
                .sig
                .inputs
                .iter()
                .filter_map(|a| match a {
                    FnArg::Receiver(_) => None,
                    FnArg::Typed(t) => Some(t),
                })
                .map(|a| &a.pat)
            {
                argument_ident_token_stream.extend(quote! { #argument_ident, })
            }

            let mut mail_fields_token_stream = proc_macro2::TokenStream::new();
            for argument_ident in method
                .sig
                .inputs
                .iter()
                .filter_map(|a| match a {
                    FnArg::Receiver(_) => None,
                    FnArg::Typed(t) => Some(t),
                })
                .map(|a| &a.pat)
            {
                mail_fields_token_stream.extend(quote! { mail.#argument_ident, })
            }

            let method_output_type = match &method.sig.output {
                syn::ReturnType::Default => quote! {()},
                syn::ReturnType::Type(_, t) => t.to_token_stream(),
            };

            let actor_method_struct = quote! {
                #[allow(non_camel_case_types)]
                pub struct #method_ident {
                    #argument_ident_type_token_stream
                }

                impl #method_ident {
                    #method_attributes_token_stream
                    pub fn new(#argument_ident_type_token_stream) -> Self {
                        Self {
                            #argument_ident_token_stream
                        }
                    }
                }

                impl crate::implementation::utils::actor::Mail for #method_ident {
                    type Result = #method_output_type;
                }

                #[async_trait::async_trait]
                impl crate::implementation::utils::actor::MailHandler<#method_ident> for #actor_type {
                    async fn handle(&mut self, mail: #method_ident) -> <#method_ident as crate::implementation::utils::actor::Mail>::Result {
                        self.#method_ident(
                            #mail_fields_token_stream
                        ).await
                    }
                }
            };

            actor_structs.extend(actor_method_struct);
        }

        actor_structs
    }

    let input = parse_macro_input!(item_token_stream as ItemImpl);
    let actor_interface_token_stream = get_actor_interface_token_stream(&input);

    quote! {
        #input

        #actor_interface_token_stream
    }
    .into()
}
