use proc_macro::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Field, FnArg, Index, ItemImpl};

#[proc_macro_derive(DdsHasKey, attributes(key))]
pub fn derive_dds_has_key(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    if let syn::Data::Struct(struct_data) = &input.data {
        let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
        let ident = input.ident;

        let has_key = struct_data.fields.iter().any(field_has_key_attribute);

        quote! {
            impl #impl_generics dust_dds::topic_definition::type_support::DdsHasKey for #ident #type_generics #where_clause {
                const HAS_KEY: bool = #has_key;
            }
        }
    } else {
        quote_spanned!{input.span() => compile_error!("DdsHasKey can only be derived for structs");}
    }
    .into()
}

#[proc_macro_derive(DdsBorrowKeyHolder, attributes(key))]
pub fn derive_dds_get_key(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    if let syn::Data::Struct(struct_data) = &input.data {
        let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
        let ident = input.ident;

        // Collect all the key fields
        let key_fields: Vec<&Field> = struct_data.fields.iter().filter(|&f|field_has_key_attribute(f)).collect();

        match key_fields.is_empty() {
            false => {

                let mut borrowed_key_holder_fields = quote!{};
                let mut borrowed_key_holder_field_assignment = quote!{};

                for key_field in key_fields {
                    let field_ident = &key_field.ident;
                    let field_type = &key_field.ty;
                    borrowed_key_holder_fields.extend(quote!{#field_ident: <#field_type as dust_dds::topic_definition::type_support::DdsBorrowKeyHolder>::BorrowedKeyHolder<'__local>,});
                    borrowed_key_holder_field_assignment.extend(quote!{#field_ident: self.#field_ident.get_key(),});

                }


                // Create the new structs and implementation inside a const to avoid name conflicts
                quote! {
                    const _ : () = {
                        #[derive(serde::Serialize)]
                        pub struct BorrowedKeyHolder<'__local> {
                            #borrowed_key_holder_fields
                        }

                        impl #impl_generics dust_dds::topic_definition::type_support::DdsBorrowKeyHolder for #ident #type_generics #where_clause {
                            type BorrowedKeyHolder<'__local> = BorrowedKeyHolder<'__local> where Self: '__local;

                            fn get_key(&self) -> Self::BorrowedKeyHolder<'_> {
                                BorrowedKeyHolder {
                                    #borrowed_key_holder_field_assignment
                                }
                            }
                        }
                    };
                }
            },
            true => {
                quote! {
                    impl #impl_generics dust_dds::topic_definition::type_support::DdsBorrowKeyHolder for #ident #type_generics #where_clause {
                        type BorrowedKeyHolder<'__local> = ();

                        fn get_key(&self) -> Self::BorrowedKeyHolder<'_> {}
                    }
                }
            }
        }
    } else {
        quote_spanned! {input.span() => compile_error!("DdsBorrowKeyHolder can only be derived for structs");}
    }
    .into()
}

#[proc_macro_derive(DdsOwningKeyHolder, attributes(key))]
pub fn derive_dds_set_key_fields(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    if let syn::Data::Struct(struct_data) = &input.data {
        let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
        let ident = input.ident;

        // Collect all the key fields
        let key_fields: Vec<&Field> = struct_data.fields.iter().filter(|&f|field_has_key_attribute(f)).collect();

        match key_fields.is_empty() {
            false => {

                let mut owning_key_holder_fields = quote!{};
                let mut set_key_fields = quote!{};

                for key_field in key_fields {
                    let field_ident = &key_field.ident;
                    let field_type = &key_field.ty;

                    owning_key_holder_fields.extend(quote!{#field_ident: <#field_type as dust_dds::topic_definition::type_support::DdsOwningKeyHolder>::OwningKeyHolder,});
                    set_key_fields.extend(quote!{ self.#field_ident.set_key_from_holder(key_holder.#field_ident);});

                }

                // Create the new structs and implementation inside a const to avoid name conflicts
                quote! {
                    const _ : () = {
                        #[derive(serde::Serialize, serde::Deserialize)]
                        pub struct OwningKeyHolder {
                            #owning_key_holder_fields
                        }

                        impl #impl_generics dust_dds::topic_definition::type_support::DdsOwningKeyHolder for #ident #type_generics #where_clause {
                            type OwningKeyHolder = OwningKeyHolder;
                        }
                    };
                }
            },
            true => {
                quote! {
                    impl #impl_generics dust_dds::topic_definition::type_support::DdsOwningKeyHolder for #ident #type_generics #where_clause {
                        type OwningKeyHolder = ();
                    }
                }
            }
        }
    } else {
        quote_spanned! {input.span() => compile_error!("DdsBorrowKeyHolder can only be derived for structs");}
    }
    .into()
}

#[proc_macro_derive(DdsRepresentation, attributes(key))]
pub fn derive_dds_representation(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    if let syn::Data::Struct(_) = &input.data {
        let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
        let ident = input.ident;

        quote! {
            impl #impl_generics dust_dds::topic_definition::type_support::DdsRepresentation for #ident #type_generics #where_clause {
                const REPRESENTATION: dust_dds::topic_definition::type_support::RtpsRepresentation
                    = dust_dds::topic_definition::type_support::RtpsRepresentation::CdrLe;
            }
        }
    }else {
        quote_spanned! {input.span() => compile_error!("DdsRepresentation can only be derived for structs");}
    }.into()
}

#[proc_macro_derive(DdsSerialize)]
pub fn derive_dds_serialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let mut field_serialization = quote!();

    if let syn::Data::Struct(struct_data) = &input.data {
        let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
        let ident = input.ident;

        let mut tuple_field_counter = 0;
        for field in struct_data.fields.iter() {
            match &field.ident {
                Some(field_name) => {
                    field_serialization.extend(quote!{self.#field_name.serialize(serializer)?;});
                },
                None => {
                    let index = Index::from(tuple_field_counter);
                    field_serialization.extend(
                        quote!{self.#index.serialize(serializer)?;}
                    );
                    tuple_field_counter+=1;
                },
            }
        }

        quote! {
            const _ : () = {
                impl #impl_generics dust_dds::topic_definition::type_support::DdsSerialize for #ident #type_generics #where_clause {
                    fn serialize(&self, serializer: &mut impl dust_dds::topic_definition::type_support::DdsSerializer) -> dust_dds::infrastructure::error::DdsResult<()> {
                        #field_serialization
                        Ok(())
                    }
                }
            };
        }
    }else {
        quote_spanned! {input.span() => compile_error!("DdsSerialize can only be derived for structs");}
    }.into()
}

#[proc_macro_derive(DdsType, attributes(key))]
pub fn derive_dds_type(input: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();

    output.extend(derive_dds_representation(input.clone()));
    output.extend(derive_dds_has_key(input.clone()));
    output.extend(derive_dds_get_key(input.clone()));
    output.extend(derive_dds_set_key_fields(input));

    output
}

fn field_has_key_attribute(field: &Field) -> bool {
    field.attrs.iter().any(|attr| attr.path().is_ident("key"))
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
