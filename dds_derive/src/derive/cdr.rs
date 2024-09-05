use super::enum_support::{get_enum_bitbound, read_enum_variant_discriminant_mapping, BitBound};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Index, Result};

fn get_discriminant_type(max_discriminant: &usize) -> TokenStream {
    match get_enum_bitbound(max_discriminant) {
        BitBound::Bit8 => quote! {u8},
        BitBound::Bit16 => quote! {u16},
        BitBound::Bit32 => quote! {u32},
    }
}

pub fn expand_cdr_deserialize(input: &DeriveInput) -> Result<TokenStream> {
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
            let mut struct_deserialization = quote!();

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
                        for field in data_struct.fields.iter() {
                            if is_field_byte_array(field) {
                                field_deserialization.extend(quote!{dust_dds::serialized_payload::cdr::deserializer::CdrDeserializer::deserialize_byte_array(deserializer)?.clone(),});
                            } else if is_field_byte_vec(field) {
                                field_deserialization.extend(quote!{dust_dds::serialized_payload::cdr::deserializer::CdrDeserializer::deserialize_bytes(deserializer)?.into(),});
                            } else {
                                field_deserialization.extend(quote!{dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize::deserialize(deserializer)?,});
                            }
                        }
                        struct_deserialization.extend(quote! {Self(#field_deserialization)})
                    } else {
                        for field in data_struct.fields.iter() {
                            let field_name = field.ident.as_ref().expect("Is not a tuple");
                            if is_field_byte_array(field) {
                                field_deserialization.extend(quote!{#field_name: dust_dds::serialized_payload::cdr::deserializer::CdrDeserializer::deserialize_byte_array(deserializer)?.clone(),});
                            } else if is_field_byte_vec(field) {
                                field_deserialization.extend(quote!{#field_name: dust_dds::serialized_payload::cdr::deserializer::CdrDeserializer::deserialize_bytes(deserializer)?.into(),});
                            } else {
                                field_deserialization.extend(quote!{#field_name: dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize::deserialize(deserializer)?,});
                            }
                        }
                        struct_deserialization.extend(quote! {Self{
                            #field_deserialization
                        }})
                    }
                }
            }

            Ok(quote! {
                    impl #generics dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize<'__de> for #ident #type_generics #where_clause {
                        fn deserialize(deserializer: &mut impl dust_dds::serialized_payload::cdr::deserializer::CdrDeserializer<'__de>) -> Result<Self, std::io::Error> {
                            Ok(#struct_deserialization)
                        }
                    }
            })
        }
        syn::Data::Enum(data_enum) => {
            let discriminant_mapping = read_enum_variant_discriminant_mapping(data_enum);

            let deserialize_enum = if discriminant_mapping.is_empty() {
                // Empty enum is the same as empty type. Do nothing
                quote! {}
            } else {
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
                let error_msg = format!("Invalid value {{}} for discriminant of {}", ident);

                quote! {
                    let discriminant : #discriminant_type = dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize::deserialize(deserializer)?;

                    match discriminant {
                        #(#clauses)*
                        _ =>  Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(#error_msg, discriminant),
                        ))
                    }
                }
            };

            Ok(quote! {
                impl #generics dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize<'__de> for #ident #type_generics #where_clause {
                    fn deserialize(deserializer: &mut impl dust_dds::serialized_payload::cdr::deserializer::CdrDeserializer<'__de>) -> Result<Self, std::io::Error> {
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

fn is_field_byte_array(field: &syn::Field) -> bool {
    if let syn::Type::Array(t) = &field.ty {
        if let syn::Type::Path(t) = t.elem.as_ref() {
            return t.path.is_ident("u8");
        }
    }
    false
}

fn is_field_byte_vec(field: &syn::Field) -> bool {
    if let syn::Type::Path(t) = &field.ty {
        if t.path.segments[0].ident == "Vec" {
            if let syn::PathArguments::AngleBracketed(a) = &t.path.segments[0].arguments {
                if let syn::GenericArgument::Type(syn::Type::Path(vec_t)) = &a.args[0] {
                    return vec_t.path.is_ident("u8");
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;
    use syn::ItemImpl;

    use super::*;


    #[test]
    fn cdr_deserialize_enum() {
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

        let output_token_stream = expand_cdr_deserialize(&input).unwrap();
        let result = syn::parse2::<ItemImpl>(output_token_stream).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            impl<'__de> dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize<'__de> for SimpleEnum {
                fn deserialize(deserializer: &mut impl dust_dds::serialized_payload::cdr::deserializer::CdrDeserializer<'__de>) -> Result<Self, std::io::Error> {
                    let discriminant: u16 = dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize::deserialize(deserializer)?;

                    match discriminant {
                        10 => Ok(SimpleEnum::a),
                        2000 => Ok(SimpleEnum::b),
                        2001 => Ok(SimpleEnum::c),
                        _ => Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(\"Invalid value {} for discriminant of SimpleEnum\", discriminant),
                        ))
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
    fn cdr_deserialize_struct_with_basic_types() {
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

        let output_token_stream = expand_cdr_deserialize(&input).unwrap();
        let result = syn::parse2::<ItemImpl>(output_token_stream).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            impl<'__de> dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize<'__de> for MyData {
                fn deserialize(deserializer: &mut impl dust_dds::serialized_payload::cdr::deserializer::CdrDeserializer<'__de>) -> Result<Self, std::io::Error> {
                    Ok(Self {
                        x: dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize::deserialize(deserializer)?,
                        y: dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize::deserialize(deserializer)?,
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
    fn cdr_deserialize_byte_array() {
        let input = syn::parse2::<DeriveInput>(
            "
            pub struct MyByteArray {
                a: [u8;10],
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let output_token_stream = expand_cdr_deserialize(&input).unwrap();
        let result = syn::parse2::<ItemImpl>(output_token_stream).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            impl<'__de> dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize<'__de> for MyByteArray {
                fn deserialize(deserializer: &mut impl dust_dds::serialized_payload::cdr::deserializer::CdrDeserializer<'__de>) -> Result<Self, std::io::Error> {
                    Ok(Self {
                        a: dust_dds::serialized_payload::cdr::deserializer::CdrDeserializer::deserialize_byte_array(deserializer)?.clone(),
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
    fn cdr_deserialize_bytes() {
        let input = syn::parse2::<DeriveInput>(
            "
            pub struct MyBytes {
                a: Vec<u8>,
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let output_token_stream = expand_cdr_deserialize(&input).unwrap();
        let result = syn::parse2::<ItemImpl>(output_token_stream).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            impl<'__de> dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize<'__de> for MyBytes {
                fn deserialize(deserializer: &mut impl dust_dds::serialized_payload::cdr::deserializer::CdrDeserializer<'__de>) -> Result<Self, std::io::Error> {
                    Ok(Self {
                        a: dust_dds::serialized_payload::cdr::deserializer::CdrDeserializer::deserialize_bytes(deserializer)?.into(),
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
    fn cdr_deserialize_struct_with_lifetime() {
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

        let result = syn::parse2::<ItemImpl>(expand_cdr_deserialize(&input).unwrap()).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            impl<'__de : 'a, 'a> dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize<'__de> for BorrowedData<'a> {
                fn deserialize(deserializer: &mut impl dust_dds::serialized_payload::cdr::deserializer::CdrDeserializer<'__de>) -> Result<Self, std::io::Error> {
                    Ok(Self {
                        data: dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize::deserialize(deserializer)?,
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
}
