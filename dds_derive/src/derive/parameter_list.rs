use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Attribute, DeriveInput, Index, Result};

fn get_parameter_attributes(
    parameter_attribute: &Attribute,
) -> syn::Result<(syn::Expr, Option<syn::Expr>, bool, bool)> {
    let mut id: Option<syn::Expr> = None;
    let mut default: Option<syn::Expr> = None;
    let mut collection = false;
    let mut skip_serialize = false;
    parameter_attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("id") {
            id = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("default") {
            default = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("collection") {
            collection = true;
            Ok(())
        } else if meta.path.is_ident("skip_serialize") {
            skip_serialize = true;
            Ok(())
        } else {
            Err(syn::Error::new(
                meta.path.span(),
                format!(
                    r#"Unexpected element {}. Valid options are "id", "default", "collection", "skip_serialize"."#,
                    meta.path.into_token_stream(),
                ),
            ))
        }
    })?;

    Ok((
        id.ok_or_else(|| {
            syn::Error::new(parameter_attribute.span(), "\"id\" attribute not found")
        })?,
        default,
        collection,
        skip_serialize,
    ))
}

pub fn expand_parameter_list_serialize(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(data_struct) => {
            let mut field_serialization = quote! {};
            let ident = &input.ident;
            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

            for (field_index, field) in data_struct.fields.iter().enumerate() {
                if let Some(parameter_attribute) =
                    field.attrs.iter().find(|a| a.path().is_ident("parameter"))
                {
                    let (id, default_value, collection, skip_serialize) =
                        get_parameter_attributes(parameter_attribute)?;

                    if !skip_serialize {
                        if !collection {
                            match (&field.ident, default_value) {
                            (Some(field_name), None) => field_serialization.extend(quote! {
                                dust_dds::serialized_payload::parameter_list::serializer::ParameterListSerializer::write(serializer, #id, &self.#field_name)?;
                            }),
                            (Some(field_name), Some(default)) => field_serialization.extend(quote! {
                                dust_dds::serialized_payload::parameter_list::serializer::ParameterListSerializer::write_with_default(serializer, #id, &self.#field_name, & #default)?;
                            }),
                            (None, None) => {
                                let index = Index::from(field_index);
                                field_serialization.extend(quote! {
                                    dust_dds::serialized_payload::parameter_list::serializer::ParameterListSerializer::write(serializer, #id, &self.#index)?;
                                })
                            }
                            (None, Some(default)) => {
                                let index = Index::from(field_index);
                                field_serialization.extend(quote! {
                                    dust_dds::serialized_payload::parameter_list::serializer::ParameterListSerializer::write_with_default(serializer, #id, &self.#index, & #default)?;
                                })
                            }
                        }
                        } else {
                            match &field.ident {
                                Some(field_name) => field_serialization.extend(quote! {
                                    dust_dds::serialized_payload::parameter_list::serializer::ParameterListSerializer::write_collection(serializer, #id, &self.#field_name)?;
                                }),
                                None => {
                                    let index = Index::from(field_index);
                                    field_serialization.extend(quote! {
                                        dust_dds::serialized_payload::parameter_list::serializer::ParameterListSerializer::write_collection(serializer, #id, &self.#index)?;
                                    })
                                }
                            }
                        }
                    }
                } else {
                    match &field.ident {
                        Some(field_name) => field_serialization.extend(quote! {
                            dust_dds::serialized_payload::parameter_list::serialize::ParameterListSerialize::serialize(&self.#field_name, serializer)?;
                        }),
                        None => {
                            let index = Index::from(field_index);
                            field_serialization.extend(quote! {
                                dust_dds::serialized_payload::parameter_list::serialize::ParameterListSerialize::serialize(&self.#index, serializer)?;
                            })
                        }
                    }
                }
            }

            Ok(quote! {
                impl #impl_generics dust_dds::serialized_payload::parameter_list::serialize::ParameterListSerialize for #ident #type_generics #where_clause {
                    fn serialize(&self, serializer: &mut impl dust_dds::serialized_payload::parameter_list::serializer::ParameterListSerializer) -> Result<(), std::io::Error> {
                        #field_serialization
                        Ok(())
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

pub fn expand_parameter_list_deserialize(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(data_struct) => {
            let mut struct_deserialization = quote!();
            let (_, type_generics, where_clause) = input.generics.split_for_impl();

            // Append the '__de lifetime to the impl generics of the struct
            let mut generics = input.generics.clone();
            generics.params = Some(syn::GenericParam::Lifetime(syn::LifetimeParam::new(
                syn::Lifetime::new("'__de", Span::call_site()),
            )))
            .into_iter()
            .chain(generics.params)
            .collect();

            let ident = &input.ident;

            match data_struct.fields.is_empty() {
                true => struct_deserialization.extend(quote! {Self}),
                false => {
                    let mut field_deserialization = quote!();
                    let is_tuple = data_struct
                        .fields
                        .iter()
                        .next()
                        .expect("Should not be empty")
                        .ident
                        .is_none();
                    for field in data_struct.fields.iter() {
                        if let Some(parameter_attribute) =
                            field.attrs.iter().find(|a| a.path().is_ident("parameter"))
                        {
                            let (id, default_value, collection, _) =
                                get_parameter_attributes(parameter_attribute)?;
                            if is_tuple {
                                if collection {
                                    field_deserialization
                                        .extend(quote! {dust_dds::serialized_payload::parameter_list::deserializer::ParameterListDeserializer::read_collection(pl_deserializer, #id)?, });
                                } else {
                                    match default_value {
                                            Some(default) => field_deserialization.extend(quote!{dust_dds::serialized_payload::parameter_list::deserializer::ParameterListDeserializer::read_with_default(pl_deserializer, #id, #default)?, }),
                                            None => field_deserialization.extend(quote!{dust_dds::serialized_payload::parameter_list::deserializer::ParameterListDeserializer::read(pl_deserializer, #id)?, }),
                                        }
                                }
                            } else {
                                let field_name =
                                    field.ident.as_ref().expect("Should have named fields");

                                if collection {
                                    field_deserialization.extend(
                                        quote! {#field_name: dust_dds::serialized_payload::parameter_list::deserializer::ParameterListDeserializer::read_collection(pl_deserializer, #id)?,},
                                    );
                                } else {
                                    match default_value {
                                    Some(default) => field_deserialization
                                        .extend(quote! {#field_name: dust_dds::serialized_payload::parameter_list::deserializer::ParameterListDeserializer::read_with_default(pl_deserializer, #id, #default)?,}),
                                    None => field_deserialization
                                        .extend(quote! {#field_name: dust_dds::serialized_payload::parameter_list::deserializer::ParameterListDeserializer::read(pl_deserializer, #id)?,}),
                                    }
                                }
                            }
                        } else if is_tuple {
                            field_deserialization.extend(quote!{dust_dds::serialized_payload::parameter_list::deserialize::ParameterListDeserialize::deserialize(pl_deserializer)?,});
                        } else {
                            let field_name =
                                field.ident.as_ref().expect("Should have named fields");
                            field_deserialization.extend(quote!{#field_name: dust_dds::serialized_payload::parameter_list::deserialize::ParameterListDeserialize::deserialize(pl_deserializer)?,});
                        }
                    }

                    if is_tuple {
                        struct_deserialization.extend(quote! {Self(#field_deserialization)})
                    } else {
                        struct_deserialization.extend(quote! {
                        Self{
                            #field_deserialization
                        }})
                    }
                }
            }

            Ok(quote! {
                    impl #generics dust_dds::serialized_payload::parameter_list::deserialize::ParameterListDeserialize<'__de> for #ident #type_generics #where_clause {
                        fn deserialize(pl_deserializer: &mut impl dust_dds::serialized_payload::parameter_list::deserializer::ParameterListDeserializer<'__de>) -> Result<Self, std::io::Error> {
                            Ok(#struct_deserialization)
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
    use quote::ToTokens;
    use syn::ItemImpl;

    use super::*;

    #[test]
    fn parameter_list_struct() {
        let input = syn::parse2::<DeriveInput>(
            "
            struct ParameterListStruct {
                #[parameter(id = 1)]
                index: u8,
                #[parameter(id = PID_DATA)]
                data: u32,
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let result =
            syn::parse2::<ItemImpl>(expand_parameter_list_serialize(&input).unwrap()).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            impl dust_dds::serialized_payload::parameter_list::serialize::ParameterListSerialize for ParameterListStruct {
                fn serialize(&self, serializer: &mut impl dust_dds::serialized_payload::parameter_list::serializer::ParameterListSerializer) -> Result<(), std::io::Error> {
                    dust_dds::serialized_payload::parameter_list::serializer::ParameterListSerializer::write(serializer, 1, &self.index)?;
                    dust_dds::serialized_payload::parameter_list::serializer::ParameterListSerializer::write(serializer, PID_DATA, &self.data)?;
                    Ok(())
                }
            }
            "
            .parse()
            .unwrap(),
        )
        .unwrap();

        assert_eq!(
            result,
            expected,
            "\n\n Result: {:?} \n\n Expected: {:?}",
            result.clone().into_token_stream().to_string(),
            expected.clone().into_token_stream().to_string(),
        );
    }

    #[test]
    fn parameter_list_struct_with_default() {
        let input = syn::parse2::<DeriveInput>(
            "
            struct ParameterListStructDefault {
                #[parameter(id = 1)]
                index: u8,
                #[parameter(id = PID_DATA)]
                data: u32,
                #[parameter(id = 3, default = \"\")]
                name: String,
                #[parameter(default = Default::default(), id = 4)]
                x: f32,
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let result =
            syn::parse2::<ItemImpl>(expand_parameter_list_serialize(&input).unwrap()).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            impl dust_dds::serialized_payload::parameter_list::serialize::ParameterListSerialize for ParameterListStructDefault {
                fn serialize(&self, serializer: &mut impl dust_dds::serialized_payload::parameter_list::serializer::ParameterListSerializer) -> Result<(), std::io::Error> {
                    dust_dds::serialized_payload::parameter_list::serializer::ParameterListSerializer::write(serializer, 1, &self.index)?;
                    dust_dds::serialized_payload::parameter_list::serializer::ParameterListSerializer::write(serializer, PID_DATA, &self.data)?;
                    dust_dds::serialized_payload::parameter_list::serializer::ParameterListSerializer::write_with_default(serializer, 3, &self.name, &\"\")?;
                    dust_dds::serialized_payload::parameter_list::serializer::ParameterListSerializer::write_with_default(serializer, 4, &self.x, &Default::default())?;
                    Ok(())
                }
            }
            "
            .parse()
            .unwrap(),
        )
        .unwrap();

        assert_eq!(
            result,
            expected,
            "\n\n Result: {:?} \n\n Expected: {:?}",
            result.clone().into_token_stream().to_string(),
            expected.clone().into_token_stream().to_string(),
        );
    }
}
