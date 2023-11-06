use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Result};

pub fn expand_cdr_deserialize(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(data_struct) => {
            let mut struct_deserialization = quote!();
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

            let ident = &input.ident;

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
                        for _ in data_struct.fields.iter() {
                            field_deserialization.extend(quote!{dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize::deserialize(deserializer)?,});
                        }
                        struct_deserialization.extend(quote! {Self(#field_deserialization)})
                    } else {
                        for field in data_struct.fields.iter() {
                            let field_name = field.ident.as_ref().expect("Is not a tuple");
                            field_deserialization.extend(quote!{#field_name: dust_dds::serialized_payload::cdr::deserialize::CdrDeserialize::deserialize(deserializer)?,});
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
    fn struct_with_basic_types() {
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
    fn struct_with_lifetime() {
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
