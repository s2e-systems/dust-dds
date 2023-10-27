use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

pub fn expand_cdr_deserialize(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(data_struct) => {
            let mut struct_deserialization = quote!();
            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

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
                            field_deserialization.extend(quote!{dust_dds::topic_definition::cdr_type::CdrDeserialize::deserialize(deserializer)?,});
                        }
                        struct_deserialization.extend(quote! {Self(#field_deserialization)})
                    } else {
                        for field in data_struct.fields.iter() {
                            let field_name = field.ident.as_ref().expect("Is not a tuple");
                            field_deserialization.extend(quote!{#field_name: dust_dds::topic_definition::cdr_type::CdrDeserialize::deserialize(deserializer)?,});
                        }
                        struct_deserialization.extend(quote! {Self{
                            #field_deserialization
                        }})
                    }
                }
            }

            Ok(quote! {
                    impl<'__de> #impl_generics dust_dds::topic_definition::cdr_type::CdrDeserialize<'__de> for #ident #type_generics #where_clause {
                        fn deserialize(deserializer: &mut impl dust_dds::topic_definition::cdr_type::CdrDeserializer<'__de>) -> dust_dds::infrastructure::error::DdsResult<Self> {
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
        println!("{:?}", output_token_stream.to_string());
        let result = syn::parse2::<ItemImpl>(output_token_stream).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            impl<'__de> dust_dds::topic_definition::cdr_type::CdrDeserialize<'__de> for MyData {
                fn deserialize(deserializer: &mut impl dust_dds::topic_definition::cdr_type::CdrDeserializer<'__de>) -> dust_dds::infrastructure::error::DdsResult<Self> {
                    Ok(Self {
                        x: dust_dds::topic_definition::cdr_type::CdrDeserialize::deserialize(deserializer)?,
                        y: dust_dds::topic_definition::cdr_type::CdrDeserialize::deserialize(deserializer)?,
                    })
                }
            }
            "
            .parse()
            .unwrap(),
        )
        .unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    #[ignore = "Lifetimes not yet handled"]
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
            impl<'__de, 'a> dust_dds::topic_definition::cdr_type::CdrDeserialize<'__de> for BorrowedData<'a> where '__de:'a {
                fn deserialize(deserializer: &mut impl dust_dds::topic_definition::cdr_type::CdrDeserializer<'__de>) -> dust_dds::infrastructure::error::DdsResult<Self> {
                    Ok(Self {
                        data: dust_dds::topic_definition::cdr_type::CdrDeserialize::deserialize(deserializer),
                    })
                }
            }
            "
            .parse()
            .unwrap(),
        )
        .unwrap();

        assert_eq!(result, expected);
    }
}
