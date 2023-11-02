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
    match &input.data {
        syn::Data::Struct(data_struct) => {
            let has_key = struct_has_key(data_struct)?;
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

            let has_key = struct_has_key(data_struct)?;

            let serialize_key_body = match has_key {
                true => {
                    let mut borrowed_key_holder_fields = quote! {};
                    let mut borrowed_key_holder_field_assignment = quote! {};

                    for field in data_struct.fields.iter() {
                        if field_has_key_attribute(field)? {
                            let field_ident = &field.ident;
                            let field_type = &field.ty;
                            borrowed_key_holder_fields
                                .extend(quote! {#field_ident: &'__borrowed #field_type,});
                            borrowed_key_holder_field_assignment
                                .extend(quote! {#field_ident: &self.#field_ident,});
                        }
                    }

                    quote! {
                        #[allow(non_camel_case_types)]
                        #[derive(dust_dds::serialized_payload::serialize::CdrSerialize)]
                        struct __borrowed_key_holder<'__borrowed> {
                            #borrowed_key_holder_fields
                        }

                        dust_dds::topic_definition::type_support::serialize_rtps_classic_cdr(
                            &__borrowed_key_holder{
                                #borrowed_key_holder_field_assignment
                            },
                            writer,
                            dust_dds::serialized_payload::endianness::CdrEndianness::LittleEndian)
                    }
                }
                false => quote! {Ok(())},
            };
            Ok(quote! {
                impl #impl_generics dust_dds::topic_definition::type_support::DdsSerializeKey for #ident #type_generics #where_clause {
                    fn serialize_key(&self, writer: impl std::io::Write) -> dust_dds::infrastructure::error::DdsResult<()> {
                        #serialize_key_body
                    }
                }
            })
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

pub fn expand_dds_instance_handle(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(data_struct) => {
            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
            let ident = &input.ident;

            let has_key = struct_has_key(data_struct)?;

            let serialize_key_body = match has_key {
                true => {
                    let mut borrowed_key_holder_fields = quote! {};
                    let mut borrowed_key_holder_field_assignment = quote! {};

                    for field in data_struct.fields.iter() {
                        if field_has_key_attribute(field)? {
                            let field_ident = &field.ident;
                            let field_type = &field.ty;
                            borrowed_key_holder_fields
                                .extend(quote! {#field_ident: &'__borrowed #field_type,});
                            borrowed_key_holder_field_assignment
                                .extend(quote! {#field_ident: &self.#field_ident,});
                        }
                    }

                    quote! {
                        #[allow(non_camel_case_types)]
                        #[derive(dust_dds::serialized_payload::serialize::CdrSerialize)]
                        struct __borrowed_key_holder<'__borrowed> {
                            #borrowed_key_holder_fields
                        }

                        let mut writer = Vec::new();
                        dust_dds::topic_definition::type_support::serialize_rtps_classic_cdr(
                        &__borrowed_key_holder{
                            #borrowed_key_holder_field_assignment
                        },
                        &mut writer,
                        dust_dds::serialized_payload::endianness::CdrEndianness::BigEndian)?;
                        Ok(dust_dds::infrastructure::instance::InstanceHandle::from(writer.as_ref()))
                    }
                }
                false => quote! {Ok(Default::default())},
            };
            Ok(quote! {
                impl #impl_generics dust_dds::topic_definition::type_support::DdsInstanceHandle for #ident #type_generics #where_clause {
                    fn get_instance_handle(&self) -> dust_dds::infrastructure::error::DdsResult<
                        dust_dds::infrastructure::instance::InstanceHandle
                    > {
                        #serialize_key_body
                    }
                }
            })
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

pub fn expand_dds_instance_handle_from_serialized_data(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(_data_struct) => {
            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
            let ident = &input.ident;

            Ok(quote! {
                impl #impl_generics dust_dds::topic_definition::type_support::DdsInstanceHandleFromSerializedData for #ident #type_generics #where_clause {
                    fn get_handle_from_serialized_data(serialized_data: &[u8]) -> dust_dds::infrastructure::error::DdsResult<
                        dust_dds::infrastructure::instance::InstanceHandle
                    > {
                        dust_dds::topic_definition::type_support::DdsInstanceHandle::get_instance_handle(
                            &<#ident as dust_dds::topic_definition::type_support::DdsDeserialize>::deserialize_data(serialized_data)?
                        )
                    }
                }
            })
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
