use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataStruct, DeriveInput, Field, Result};

fn field_has_key_attribute(field: &Field) -> bool {
    field.attrs.iter().any(|attr| attr.path().is_ident("key"))
}

fn struct_has_key(data_struct: &DataStruct) -> bool {
    data_struct.fields.iter().any(field_has_key_attribute)
}

pub fn expand_has_key(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(data_struct) => {
            let has_key = struct_has_key(data_struct);
            let ident = &input.ident;
            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
            Ok(quote!(
                impl #impl_generics dust_dds::topic_definition::type_support::DdsHasKey for #ident #type_generics #where_clause {
                    const HAS_KEY: bool = #has_key;
                }
            ))
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

pub fn expand_dds_serialize_key(input: &DeriveInput) -> Result<TokenStream> {
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

#[cfg(test)]
mod tests {
    use syn::ItemImpl;

    use super::*;

    #[test]
    fn struct_with_key() {
        let input = syn::parse2::<DeriveInput>(
            "
            struct WithKey {
                #[key]
                id: u8
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let result = syn::parse2::<ItemImpl>(expand_has_key(&input).unwrap()).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            impl dust_dds::topic_definition::type_support::DdsHasKey for WithKey {
                const HAS_KEY: bool = true;
            }"
            .parse()
            .unwrap(),
        )
        .unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn struct_without_key() {
        let input = syn::parse2::<DeriveInput>(
            "
            struct WithoutKey {
                data: u32
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let result = syn::parse2::<ItemImpl>(expand_has_key(&input).unwrap()).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            impl dust_dds::topic_definition::type_support::DdsHasKey for WithoutKey {
                const HAS_KEY: bool = false;
            }"
            .parse()
            .unwrap(),
        )
        .unwrap();

        assert_eq!(result, expected);
    }
}
