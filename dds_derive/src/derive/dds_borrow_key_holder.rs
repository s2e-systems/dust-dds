use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Field, Result};

use crate::attribute_helpers::field_has_key_attribute;

pub fn expand_dds_borrow_key_holder(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(data_struct) => {
            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
            let ident = &input.ident;

            // Collect all the key fields
            let key_fields: Vec<&Field> = data_struct
                .fields
                .iter()
                .filter(|&f| field_has_key_attribute(f))
                .collect();

            match key_fields.is_empty() {
                false => {
                    let mut borrowed_key_holder_fields = quote! {};
                    let mut borrowed_key_holder_field_assignment = quote! {};

                    for key_field in key_fields {
                        let field_ident = &key_field.ident;
                        let field_type = &key_field.ty;
                        borrowed_key_holder_fields.extend(quote!{#field_ident: <#field_type as dust_dds::topic_definition::type_support::DdsBorrowKeyHolder>::BorrowedKeyHolder<'__local>,});
                        borrowed_key_holder_field_assignment
                            .extend(quote! {#field_ident: self.#field_ident.get_key(),});
                    }

                    // Create the new structs and implementation inside a const to avoid name conflicts
                    Ok(quote! {
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
                    })
                }
                true => Ok(quote! {
                    impl #impl_generics dust_dds::topic_definition::type_support::DdsBorrowKeyHolder for #ident #type_generics #where_clause {
                        type BorrowedKeyHolder<'__local> = ();

                        fn get_key(&self) -> Self::BorrowedKeyHolder<'_> {}
                    }
                }),
            }
        }
        syn::Data::Enum(data_enum) => Err(syn::Error::new(
            data_enum.enum_token.span,
            "Enum not supported",
        )),
        syn::Data::Union(data_union) => Err(syn::Error::new(
            data_union.union_token.span,
            "Union not supported",
        )),
    }
}
