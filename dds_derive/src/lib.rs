use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Field};

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

#[proc_macro_derive(DdsGetKey, attributes(key))]
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
                    borrowed_key_holder_fields.extend(quote!{#field_ident: <#field_type as dust_dds::topic_definition::type_support::DdsGetKey>::BorrowedKeyHolder<'a>,});
                    borrowed_key_holder_field_assignment.extend(quote!{#field_ident: self.#field_ident.get_key(),});

                }


                // Create the new structs and implementation inside a const to avoid name conflicts
                quote! {
                    const _ : () = {
                        #[derive(serde::Serialize)]
                        pub struct BorrowedKeyHolder<'a> {
                            #borrowed_key_holder_fields
                        }

                        impl #impl_generics dust_dds::topic_definition::type_support::DdsGetKey for #ident #type_generics #where_clause {
                            type BorrowedKeyHolder<'a> = BorrowedKeyHolder<'a>;

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
                    impl #impl_generics dust_dds::topic_definition::type_support::DdsGetKey for #ident #type_generics #where_clause {
                        type BorrowedKeyHolder<'a> = ();

                        fn get_key(&self) -> Self::BorrowedKeyHolder<'_> {}
                    }
                }
            }
        }
    } else {
        quote_spanned! {input.span() => compile_error!("DdsGetKey can only be derived for structs");}
    }
    .into()
}

#[proc_macro_derive(DdsSetKeyFields, attributes(key))]
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

                    owning_key_holder_fields.extend(quote!{#field_ident: <#field_type as dust_dds::topic_definition::type_support::DdsSetKeyFields>::OwningKeyHolder,});
                    set_key_fields.extend(quote!{ self.#field_ident.set_key_from_holder(key_holder.#field_ident);});

                }

                // Create the new structs and implementation inside a const to avoid name conflicts
                quote! {
                    const _ : () = {
                        #[derive(serde::Deserialize)]
                        pub struct OwningKeyHolder {
                            #owning_key_holder_fields
                        }

                        impl #impl_generics dust_dds::topic_definition::type_support::DdsSetKeyFields for #ident #type_generics #where_clause {
                            type OwningKeyHolder = OwningKeyHolder;

                            fn set_key_from_holder(&mut self, key_holder: Self::OwningKeyHolder) {
                                #set_key_fields
                            }
                        }
                    };
                }
            },
            true => {
                quote! {
                    impl #impl_generics dust_dds::topic_definition::type_support::DdsSetKeyFields for #ident #type_generics #where_clause {
                        type OwningKeyHolder = ();

                        fn set_key_from_holder(&mut self, key_holder: Self::OwningKeyHolder) {}
                    }
                }
            }
        }
    } else {
        quote_spanned! {input.span() => compile_error!("DdsGetKey can only be derived for structs");}
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
                const REPRESENTATION_IDENTIFIER: dust_dds::topic_definition::type_support::RepresentationType
                    = dust_dds::topic_definition::type_support::RepresentationType::CdrLe;
            }
        }
    }else {
        quote_spanned! {input.span() => compile_error!("DdsRepresentation can only be derived for structs");}
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
    field.attrs.iter().any(|attr| {
        attr.parse_meta()
            .ok()
            .and_then(|meta| meta.path().get_ident().cloned())
            .map(|ident| ident == "key")
            .unwrap_or(false)
    })
}
