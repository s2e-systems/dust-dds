mod derive;

use derive::{
    cdr::{expand_cdr_deserialize, expand_cdr_serialize},
    dds_key::{expand_dds_key, expand_has_key},
    dds_serialize_data::{expand_dds_deserialize_data, expand_dds_serialize_data},
    dds_type_xml::expand_dds_type_xml,
    parameter_list::{expand_parameter_list_deserialize, expand_parameter_list_serialize},
};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, FnArg, ItemImpl};

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

#[proc_macro_derive(ParameterListSerialize, attributes(parameter))]
pub fn derive_parameter_list_serialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_parameter_list_serialize(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(ParameterListDeserialize, attributes(parameter))]
pub fn derive_parameter_list_deserialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_parameter_list_deserialize(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DdsSerialize, attributes(dust_dds))]
pub fn derive_dds_serialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_dds_serialize_data(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DdsDeserialize, attributes(dust_dds))]
pub fn derive_dds_deserialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_dds_deserialize_data(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DdsHasKey, attributes(dust_dds))]
pub fn derive_dds_has_key(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_has_key(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DdsKey, attributes(dust_dds))]
pub fn derive_dds_key(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_dds_key(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DdsTypeXml, attributes(dust_dds))]
pub fn derive_dds_type_xml(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_dds_type_xml(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DdsType, attributes(dust_dds))]
pub fn derive_dds_type(input: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();

    output.extend(derive_cdr_serialize(input.clone()));
    output.extend(derive_cdr_deserialize(input.clone()));
    output.extend(derive_dds_serialize(input.clone()));
    output.extend(derive_dds_deserialize(input.clone()));
    output.extend(derive_dds_key(input.clone()));
    output.extend(derive_dds_has_key(input.clone()));
    output.extend(derive_dds_type_xml(input));

    output
}

/// Attribute macro to generate the actor interface from
#[proc_macro_attribute]
pub fn actor_interface(
    _attribute_token_stream: TokenStream,
    item_token_stream: TokenStream,
) -> TokenStream {
    fn create_actor_handler_token_stream(input: &ItemImpl) -> proc_macro2::TokenStream {
        let actor_type = match input.self_ty.as_ref() {
            syn::Type::Path(p) => p,
            _ => panic!("Expect impl block with type"),
        };

        let actor_ident = match actor_type.path.get_ident() {
            Some(i) => i,
            None => panic!("Expected path with ident"),
        };
        let actor_message_enum_ident =
            Ident::new(&format!("{}MessageKind", actor_ident), Span::call_site());

        let mut enum_variants: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut enum_handle_variants: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut actor_method_variants: Vec<proc_macro2::TokenStream> = Vec::new();

        for method in input.items.iter().filter_map(|i| match i {
            syn::ImplItem::Fn(m) => Some(m),
            _ => None,
        }) {
            let method_ident = &method.sig.ident;
            let (methods_arguments_ident, methods_arguments_type): (Vec<_>, Vec<_>) = method
                .sig
                .inputs
                .iter()
                .filter_map(|a| match a {
                    FnArg::Receiver(_) => None,
                    FnArg::Typed(t) => Some(t),
                })
                .map(|a| (a.pat.as_ref(), a.ty.as_ref()))
                .unzip();

            let method_await = if method.sig.asyncness.is_some() {
                quote! {.await}
            } else {
                quote! {}
            };

            match &method.sig.output {
                syn::ReturnType::Default => {
                    let enum_variant = quote! {
                        #method_ident{
                            #(#methods_arguments_ident: #methods_arguments_type, )*
                        }
                    };
                    let enum_handle_variant = quote! {
                        #actor_message_enum_ident::#method_ident{
                            #(#methods_arguments_ident, )*
                        } => {
                            self.#method_ident(#(#methods_arguments_ident, )*)#method_await;
                        }
                    };

                    let actor_method_variant = quote! {
                        #[allow(clippy::too_many_arguments, clippy::unused_unit)]
                        pub async fn #method_ident(&self, #(#methods_arguments_ident: #methods_arguments_type, )*){
                            let message = #actor_message_enum_ident::#method_ident{
                                #(#methods_arguments_ident, )*
                            };
                            self.send_actor_message(message).await
                        }
                    };

                    enum_variants.push(enum_variant);
                    enum_handle_variants.push(enum_handle_variant);
                    actor_method_variants.push(actor_method_variant);
                }
                syn::ReturnType::Type(_, output_type) => {
                    let enum_variant = quote! {
                        #method_ident{
                            #(#methods_arguments_ident: #methods_arguments_type, )*
                            __response_sender: tokio::sync::oneshot::Sender<#output_type>,
                        }
                    };

                    let enum_handle_variant = quote! {
                        #actor_message_enum_ident::#method_ident{
                            #(#methods_arguments_ident, )*
                            __response_sender,
                        } => {
                            let r = self.#method_ident(#(#methods_arguments_ident, )*)#method_await;
                            __response_sender.send(r).ok();
                        }
                    };

                    let actor_method_variant = quote! {
                        #[allow(clippy::too_many_arguments, clippy::unused_unit)]
                        pub async fn #method_ident(&self, #(#methods_arguments_ident: #methods_arguments_type, )*) -> #output_type {
                            let (__response_sender, response_receiver) = tokio::sync::oneshot::channel();
                            let message = #actor_message_enum_ident::#method_ident{
                                #(#methods_arguments_ident, )*
                                __response_sender,
                            };
                            self.send_actor_message(message).await;
                            response_receiver.await.expect("Message is always processed as long as actor object exists")
                        }
                    };

                    enum_variants.push(enum_variant);
                    enum_handle_variants.push(enum_handle_variant);
                    actor_method_variants.push(actor_method_variant);
                }
            };
        }

        quote! {
            #[allow(non_camel_case_types)]
            pub enum #actor_message_enum_ident {
                #(#enum_variants,)*
            }

            impl crate::implementation::utils::actor::ActorHandler for #actor_ident {
                type Message = #actor_message_enum_ident;

                async fn handle_message(&mut self, message: Self::Message) {
                    match message {
                        #(#enum_handle_variants,)*
                    }
                }
            }

            impl crate::implementation::utils::actor::Actor<#actor_ident> {
                #(#actor_method_variants)*
            }
        }
    }

    let input = parse_macro_input!(item_token_stream as ItemImpl);
    let actor_handler_token_stream = create_actor_handler_token_stream(&input);

    quote! {
        #input

        #actor_handler_token_stream
    }
    .into()
}
