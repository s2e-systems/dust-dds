use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, DataStruct, DeriveInput, Field, Result};

fn field_has_key_attribute(field: &Field) -> syn::Result<bool> {
    let mut has_key = false;
    if let Some(dust_dds_attribute) = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("dust_dds"))
    {
        dust_dds_attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("key") {
                has_key = true;
                Ok(())
            } else {
                Err(syn::Error::new(
                    meta.path.span(),
                    format!(
                        "Unexpected element {}. Valid options are \"key\"",
                        meta.path.into_token_stream(),
                    ),
                ))
            }
        })?;
    }
    Ok(has_key)
}

fn struct_has_key(data_struct: &DataStruct) -> Result<bool> {
    for field in data_struct.fields.iter() {
        if field_has_key_attribute(field)? {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn expand_has_key(input: &DeriveInput) -> Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    match &input.data {
        syn::Data::Struct(data_struct) => {
            let has_key = struct_has_key(data_struct)?;
            Ok(quote!(
                impl #impl_generics dust_dds::topic_definition::type_support::DdsHasKey for #ident #type_generics #where_clause {
                    const HAS_KEY: bool = #has_key;
                }
            ))
        }
        syn::Data::Enum(_data_enum) => Ok(quote!(
            impl #impl_generics dust_dds::topic_definition::type_support::DdsHasKey for #ident #type_generics #where_clause {
                const HAS_KEY: bool = false;
            }
        )),
        syn::Data::Union(data_union) => Err(syn::Error::new(
            data_union.union_token.span,
            "Union not supported",
        )),
    }
}

pub fn expand_dds_key(input: &DeriveInput) -> Result<TokenStream> {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;
    match &input.data {
        syn::Data::Struct(data_struct) => {
            let has_key = struct_has_key(data_struct)?;

            let (key_holder_struct_definition, key_holder_struct_construction) = match has_key {
                true => {
                    let mut key_holder_fields = quote! {};
                    let mut key_holder_field_assignment = quote! {};

                    for field in data_struct.fields.iter() {
                        if field_has_key_attribute(field)? {
                            let field_ident = &field.ident;
                            let field_type = &field.ty;
                            key_holder_fields.extend(quote! {#field_ident: #field_type,});
                            key_holder_field_assignment
                                .extend(quote! {#field_ident: self.#field_ident.clone(),});
                        }
                    }

                    let key_holder_struct_definition = quote! {
                        #[allow(non_camel_case_types)]
                        #[derive(dust_dds::serialized_payload::cdr::serialize::CdrSerialize, dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize)]
                        pub struct __key_holder {
                            #key_holder_fields
                        }
                    };

                    let key_holder_struct_construction = quote! {
                        __key_holder{
                            #key_holder_field_assignment
                        }
                    };

                    (key_holder_struct_definition, key_holder_struct_construction)
                }
                false => {
                    let key_holder_struct_definition = quote! {
                        #[allow(non_camel_case_types)]
                        #[derive(dust_dds::serialized_payload::cdr::serialize::CdrSerialize, dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize)]
                        pub struct __key_holder;
                    };

                    let key_holder_struct_construction = quote! {
                        __key_holder
                    };

                    (key_holder_struct_definition, key_holder_struct_construction)
                }
            };
            Ok(quote! {
                const _ : () = {

                    #key_holder_struct_definition

                    impl #impl_generics dust_dds::topic_definition::type_support::DdsKey for #ident #type_generics #where_clause {
                        type Key = __key_holder;

                        fn get_key(&self) -> dust_dds::infrastructure::error::DdsResult<Self::Key> {
                            Ok(#key_holder_struct_construction)
                        }

                        fn get_key_from_serialized_data(serialized_foo: &[u8]) -> dust_dds::infrastructure::error::DdsResult<Self::Key> {
                            <#ident as dust_dds::topic_definition::type_support::DdsDeserialize>::deserialize_data(serialized_foo)?
                                .get_key()
                        }
                    }
                };
            })
        }
        syn::Data::Enum(_data_enum) => Ok(quote! {
            const _ : () = {
                impl #impl_generics dust_dds::topic_definition::type_support::DdsKey for #ident #type_generics #where_clause {
                    type Key = ();

                    fn get_key(&self) -> dust_dds::infrastructure::error::DdsResult<Self::Key> {
                        Ok(())
                    }

                    fn get_key_from_serialized_data(serialized_foo: &[u8]) -> dust_dds::infrastructure::error::DdsResult<Self::Key> {
                        Ok(())
                    }
                }
            };
        }),
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
    fn struct_with_key_field_should_be_has_key_true() {
        let input = syn::parse2::<DeriveInput>(
            "
            struct WithKey {
                #[dust_dds(key)]
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
    fn struct_without_key_field_should_be_has_key_false() {
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
