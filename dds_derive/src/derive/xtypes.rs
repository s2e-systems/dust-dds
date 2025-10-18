use super::{
    attributes::{Extensibility, get_field_attributes, get_input_extensibility},
    enum_support::{
        BitBound, get_enum_bitbound, is_enum_xtypes_union, read_enum_variant_discriminant_mapping,
    },
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Fields, Index, Result, spanned::Spanned};

fn get_discriminant_type(max_discriminant: &usize) -> TokenStream {
    match get_enum_bitbound(max_discriminant) {
        BitBound::Bit8 => quote! {u8},
        BitBound::Bit16 => quote! {u16},
        BitBound::Bit32 => quote! {u32},
    }
}

pub fn expand_xtypes_deserialize(input: &DeriveInput) -> Result<TokenStream> {
    let ident = &input.ident;
    let (_, type_generics, where_clause) = input.generics.split_for_impl();

    // Create a '__de lifetime bound to all the lifetimes of the struct
    let mut de_lifetime_param =
        syn::LifetimeParam::new(syn::Lifetime::new("'__de", Span::call_site()));
    for struct_lifetime in input.generics.lifetimes().cloned() {
        de_lifetime_param.bounds.push(struct_lifetime.lifetime);
    }

    // Append the '__de lifetime to the impl generics of the struct
    let mut generics = input.generics.clone();
    generics.params = Some(syn::GenericParam::Lifetime(de_lifetime_param))
        .into_iter()
        .chain(generics.params)
        .collect();

    match &input.data {
        syn::Data::Struct(data_struct) => {
            let extensibility = get_input_extensibility(input)?;
            let mut struct_deserialization = quote!();
            let deserializer_definition = match extensibility {
                Extensibility::Final => {
                    quote! {let mut d =  dust_dds::xtypes::deserializer::XTypesDeserializer::deserialize_final_struct(deserializer)?;}
                }
                Extensibility::Appendable => {
                    quote! {let mut d =  dust_dds::xtypes::deserializer::XTypesDeserializer::deserialize_appendable_struct(deserializer)?;}
                }
                Extensibility::Mutable => {
                    quote! {let mut d =  dust_dds::xtypes::deserializer::XTypesDeserializer::deserialize_mutable_struct(deserializer)?;}
                }
            };

            match data_struct.fields.is_empty() {
                true => struct_deserialization.extend(quote! {Self}),
                false => {
                    let mut field_deserialization = quote!();
                    let is_tuple = data_struct
                        .fields
                        .iter()
                        .next()
                        .expect("Not empty")
                        .ident
                        .is_none();
                    if is_tuple {
                        for (index, field) in data_struct.fields.iter().enumerate() {
                            let index_str = format!("{index:?}",);
                            match extensibility {
                                Extensibility::Final => field_deserialization
                                    .extend(quote! { dust_dds::xtypes::deserializer::DeserializeFinalStruct::deserialize_field(&mut d, #index_str)?,}),
                                Extensibility::Appendable => field_deserialization
                                    .extend(quote! { dust_dds::xtypes::deserializer::DeserializeAppendableStruct::deserialize_field(&mut d, #index_str)?,}),
                                Extensibility::Mutable => {
                                    let id = get_field_attributes(field)?.id.ok_or(syn::Error::new(field.span(), "Mutable struct must define id attribute for every field"))?;
                                    field_deserialization
                                        .extend(quote! { dust_dds::xtypes::deserializer::DeserializeMutableStruct::deserialize_field(&mut d, #id, #index_str)?,});
                                }
                            }
                        }
                        struct_deserialization.extend(quote! {Self(#field_deserialization)})
                    } else {
                        for field in data_struct.fields.iter() {
                            let field_name = field.ident.as_ref().expect("Is not a tuple");
                            let field_name_str = field_name.to_string();
                            match extensibility {
                                Extensibility::Final => field_deserialization.extend(
                                    quote! {#field_name:  dust_dds::xtypes::deserializer::DeserializeFinalStruct::deserialize_field(&mut d, #field_name_str)?,},
                                ),
                                Extensibility::Appendable => field_deserialization.extend(
                                    quote! {#field_name:  dust_dds::xtypes::deserializer::DeserializeAppendableStruct::deserialize_field(&mut d, #field_name_str)?,},
                                ),
                                Extensibility::Mutable => {
                                    let id = get_field_attributes(field)?.id.ok_or(syn::Error::new(field.span(), "Mutable struct must define id attribute for every field"))?;
                                    field_deserialization.extend(
                                        quote! {#field_name:  dust_dds::xtypes::deserializer::DeserializeMutableStruct::deserialize_field(&mut d, #id, #field_name_str)?,},
                                    );
                                }
                            }
                        }
                        struct_deserialization.extend(quote! {Self{
                            #field_deserialization
                        }})
                    }
                }
            }

            Ok(quote! {
                    #[automatically_derived]
                    impl #generics dust_dds::xtypes::deserialize::XTypesDeserialize<'__de> for #ident #type_generics #where_clause {
                        fn deserialize(deserializer: impl dust_dds::xtypes::deserializer::XTypesDeserializer<'__de>) -> Result<Self, dust_dds::xtypes::error::XTypesError> {
                            #deserializer_definition
                            Ok(#struct_deserialization)
                        }
                    }
            })
        }
        syn::Data::Enum(data_enum) => {
            let deserialize_enum = if data_enum.variants.is_empty() {
                // Empty enum is the same as empty type. Do nothing
                quote! {}
            } else {
                // Separate between Unions and Enumeration which are both
                // mapped as Rust enum types
                if is_enum_xtypes_union(data_enum) {
                    let mut variant_deserialization = quote!();
                    for variant in data_enum.variants.iter() {
                        let variant_discriminant = &variant
                            .discriminant
                            .as_ref()
                            .ok_or(syn::Error::new(
                                variant.span(),
                                "Union variant must have explicit discriminant",
                            ))?
                            .1;

                        let variant_ident = &variant.ident;
                        match &variant.fields {
                            Fields::Named(f) => {
                                let mut field_names = quote!();
                                let mut field_deserialization = quote!();
                                for field in &f.named {
                                    let field_ident = field.ident.as_ref().expect("Must be named");
                                    let field_ident_str = field_ident.to_string();
                                    field_names.extend(quote!{#field_ident,});
                                    field_deserialization.extend(quote!{
                                        let #field_ident = dust_dds::xtypes::deserializer::DeserializeFinalStruct::deserialize_field(&mut d, #field_ident_str)?;
                                    })
                                }
                                variant_deserialization.extend(quote! {

                                    #variant_discriminant => {
                                        #field_deserialization
                                        Ok(#ident::#variant_ident{#field_names})
                                    },
                                })
                            },
                            Fields::Unnamed(_) => variant_deserialization.extend(quote! {
                                #variant_discriminant => {
                                    let f = dust_dds::xtypes::deserializer::DeserializeFinalStruct::deserialize_field(&mut d, "0")?;
                                    Ok(#ident::#variant_ident(f))
                                },
                            }),
                            Fields::Unit => variant_deserialization.extend(quote! {
                                #variant_discriminant => Ok(#ident::#variant_ident),
                            }),
                        }
                    }

                    quote! {
                        let mut d =  dust_dds::xtypes::deserializer::XTypesDeserializer::deserialize_final_struct(deserializer)?;
                        let discriminator : u8 = dust_dds::xtypes::deserializer::DeserializeFinalStruct::deserialize_field(&mut d, "discriminator")?;

                        match discriminator {
                            #variant_deserialization
                            _ =>  Err(dust_dds::xtypes::error::XTypesError::InvalidData),
                        }
                    }
                } else {
                    let discriminant_mapping = read_enum_variant_discriminant_mapping(data_enum);
                    let max_discriminant = discriminant_mapping
                        .iter()
                        .map(|(_, v)| v)
                        .max()
                        .expect("Map contains at least a value");
                    let discriminant_type = get_discriminant_type(max_discriminant);
                    let clauses: Vec<_> = discriminant_mapping
                        .iter()
                        .map(|(v, d)| {
                            let i = Index::from(*d);
                            quote! {#i => Ok(#ident::#v),}
                        })
                        .collect();

                    quote! {
                        let discriminant : #discriminant_type =  dust_dds::xtypes::deserialize::XTypesDeserialize::deserialize(deserializer)?;

                        match discriminant {
                            #(#clauses)*
                            _ =>  Err(dust_dds::xtypes::error::XTypesError::InvalidData)
                        }
                    }
                }
            };

            Ok(quote! {
                #[automatically_derived]
                impl #generics dust_dds::xtypes::deserialize::XTypesDeserialize<'__de> for #ident #type_generics #where_clause {
                    fn deserialize(deserializer: impl dust_dds::xtypes::deserializer::XTypesDeserializer<'__de>) -> Result<Self, dust_dds::xtypes::error::XTypesError> {
                        #deserialize_enum
                    }
                }
            })
        }
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
    fn xtypes_deserialize_final_struct_with_basic_types() {
        let input = syn::parse2::<DeriveInput>(
            "
            struct MyData {
                x: u32,
                y: u32,
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let output_token_stream = expand_xtypes_deserialize(&input).unwrap();
        let result = syn::parse2::<ItemImpl>(output_token_stream).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            #[automatically_derived]
            impl<'__de> dust_dds::xtypes::deserialize::XTypesDeserialize<'__de> for MyData {
                fn deserialize(deserializer: impl dust_dds::xtypes::deserializer::XTypesDeserializer<'__de>) -> Result<Self, dust_dds::xtypes::error::XTypesError> {
                    let mut d = dust_dds::xtypes::deserializer::XTypesDeserializer::deserialize_final_struct(deserializer)?;
                    Ok(Self {
                        x: dust_dds::xtypes::deserializer::DeserializeFinalStruct::deserialize_field(&mut d, \"x\")?,
                        y: dust_dds::xtypes::deserializer::DeserializeFinalStruct::deserialize_field(&mut d, \"y\")?,
                    })
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
            "\n R: {:?} \n \n L: {:?} \n ",
            result.clone().into_token_stream().to_string(),
            expected.clone().into_token_stream().to_string()
        );
    }

    #[test]
    fn xtypes_deserialize_appendable_struct_with_basic_types() {
        let input = syn::parse2::<DeriveInput>(
            "
            #[dust_dds(extensibility = \"appendable\")]
            struct MyData {
                x: u32,
                y: u32,
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let output_token_stream = expand_xtypes_deserialize(&input).unwrap();
        let result = syn::parse2::<ItemImpl>(output_token_stream).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            #[automatically_derived]
            impl<'__de> dust_dds::xtypes::deserialize::XTypesDeserialize<'__de> for MyData {
                fn deserialize(deserializer: impl dust_dds::xtypes::deserializer::XTypesDeserializer<'__de>) -> Result<Self, dust_dds::xtypes::error::XTypesError> {
                    let mut d = dust_dds::xtypes::deserializer::XTypesDeserializer::deserialize_appendable_struct(deserializer)?;
                    Ok(Self {
                        x: dust_dds::xtypes::deserializer::DeserializeAppendableStruct::deserialize_field(&mut d, \"x\")?,
                        y: dust_dds::xtypes::deserializer::DeserializeAppendableStruct::deserialize_field(&mut d, \"y\")?,
                    })
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
            "\n R: {:?} \n \n L: {:?} \n ",
            result.clone().into_token_stream().to_string(),
            expected.clone().into_token_stream().to_string()
        );
    }

    #[test]
    fn xtypes_deserialize_final_struct_with_lifetime() {
        let input = syn::parse2::<DeriveInput>(
            "
            struct BorrowedData<'a> {
                data: &'a [u8]
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let result = syn::parse2::<ItemImpl>(expand_xtypes_deserialize(&input).unwrap()).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            #[automatically_derived]
            impl<'__de : 'a, 'a> dust_dds::xtypes::deserialize::XTypesDeserialize<'__de> for BorrowedData<'a> {
                fn deserialize(deserializer: impl dust_dds::xtypes::deserializer::XTypesDeserializer<'__de>) -> Result<Self, dust_dds::xtypes::error::XTypesError> {
                    let mut d = dust_dds::xtypes::deserializer::XTypesDeserializer::deserialize_final_struct(deserializer)?;
                    Ok(Self {
                        data: dust_dds::xtypes::deserializer::DeserializeFinalStruct::deserialize_field(&mut d, \"data\")?,
                    })
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
            "\n\n L: {} \n\n R: {}",
            result.clone().into_token_stream().to_string(),
            expected.clone().into_token_stream().to_string()
        );
    }

    #[test]
    fn xtypes_deserialize_mutable_struct_with_basic_types() {
        let input = syn::parse2::<DeriveInput>(
            "
            #[dust_dds(extensibility = \"mutable\")]
            struct MyData {
                #[dust_dds(id=1)]
                x: u32,
                #[dust_dds(id=2)]
                y: u32,
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let output_token_stream = expand_xtypes_deserialize(&input).unwrap();
        let result = syn::parse2::<ItemImpl>(output_token_stream).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            #[automatically_derived]
            impl<'__de> dust_dds::xtypes::deserialize::XTypesDeserialize<'__de> for MyData {
                fn deserialize(deserializer: impl dust_dds::xtypes::deserializer::XTypesDeserializer<'__de>) -> Result<Self, dust_dds::xtypes::error::XTypesError> {
                    let mut d = dust_dds::xtypes::deserializer::XTypesDeserializer::deserialize_mutable_struct(deserializer)?;
                    Ok(Self {
                        x: dust_dds::xtypes::deserializer::DeserializeMutableStruct::deserialize_field(&mut d, 1, \"x\")?,
                        y: dust_dds::xtypes::deserializer::DeserializeMutableStruct::deserialize_field(&mut d, 2, \"y\")?,
                    })
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
            "\n R: {:?} \n \n L: {:?} \n ",
            result.clone().into_token_stream().to_string(),
            expected.clone().into_token_stream().to_string()
        );
    }

    #[test]
    fn xtypes_deserialize_enum() {
        let input = syn::parse2::<DeriveInput>(
            "
            enum SimpleEnum {
                a=10,
                b=2000,
                c,
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let output_token_stream = expand_xtypes_deserialize(&input).unwrap();
        let result = syn::parse2::<ItemImpl>(output_token_stream).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            #[automatically_derived]
            impl<'__de> dust_dds::xtypes::deserialize::XTypesDeserialize<'__de> for SimpleEnum {
                fn deserialize(deserializer: impl dust_dds::xtypes::deserializer::XTypesDeserializer<'__de>) -> Result<Self, dust_dds::xtypes::error::XTypesError> {
                    let discriminant: u16 = dust_dds::xtypes::deserialize::XTypesDeserialize::deserialize(deserializer)?;

                    match discriminant {
                        10 => Ok(SimpleEnum::a),
                        2000 => Ok(SimpleEnum::b),
                        2001 => Ok(SimpleEnum::c),
                        _ => Err(dust_dds::xtypes::error::XTypesError::InvalidData)
                    }
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
            "\n R: {:?} \n \n L: {:?} \n ",
            result.clone().into_token_stream().to_string(),
            expected.clone().into_token_stream().to_string()
        );
    }

    #[test]
    fn xtypes_deserialize_enum_with_field_variants() {
        let input = syn::parse2::<DeriveInput>(
            "
            enum SimpleEnum {
                a(u32)=10,
                b{a:u32, b:i32, c:f32}=200,
                c=201,
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let output_token_stream = expand_xtypes_deserialize(&input).unwrap();
        let result = syn::parse2::<ItemImpl>(output_token_stream).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            #[automatically_derived]
            impl<'__de> dust_dds::xtypes::deserialize::XTypesDeserialize<'__de> for SimpleEnum {
                fn deserialize(deserializer: impl dust_dds::xtypes::deserializer::XTypesDeserializer<'__de>) -> Result<Self, dust_dds::xtypes::error::XTypesError> {
                    let mut d =  dust_dds::xtypes::deserializer::XTypesDeserializer::deserialize_final_struct(deserializer)?;
                    let discriminator : u8 = dust_dds::xtypes::deserializer::DeserializeFinalStruct::deserialize_field(&mut d, \"discriminator\")?;

                    match discriminator {
                        10 => {
                           let f = dust_dds::xtypes::deserializer::DeserializeFinalStruct::deserialize_field(&mut d, \"0\")?;
                           Ok(SimpleEnum::a(f))
                        },
                        200 => {
                            let a = dust_dds::xtypes::deserializer::DeserializeFinalStruct::deserialize_field(&mut d, \"a\")?;
                            let b = dust_dds::xtypes::deserializer::DeserializeFinalStruct::deserialize_field(&mut d, \"b\")?;
                            let c = dust_dds::xtypes::deserializer::DeserializeFinalStruct::deserialize_field(&mut d, \"c\")?;
                            Ok(SimpleEnum::b{a,b,c,})
                        },
                        201 => Ok(SimpleEnum::c),
                        _ => Err(dust_dds::xtypes::error::XTypesError::InvalidData),
                    }
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
            "\n R: {:?} \n \n L: {:?} \n ",
            result.clone().into_token_stream().to_string(),
            expected.clone().into_token_stream().to_string()
        );
    }
}
