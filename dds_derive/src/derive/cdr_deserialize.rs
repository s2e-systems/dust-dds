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
                const _ : () = {
                    impl<'__de> #impl_generics dust_dds::topic_definition::cdr_type::CdrDeserialize<'__de> for #ident #type_generics #where_clause {
                        fn deserialize(deserializer: &mut impl dust_dds::topic_definition::cdr_type::CdrDeserializer<'__de>) -> dust_dds::infrastructure::error::DdsResult<Self> {
                            Ok(#struct_deserialization)
                        }
                    }
                };
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
