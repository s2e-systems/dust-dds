use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Result};

use crate::attribute_helpers::get_parameter_attributes;

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
                            let (id, default_value, read_all_elements) =
                                get_parameter_attributes(parameter_attribute)?;
                            if is_tuple {
                                if read_all_elements {
                                    field_deserialization
                                        .extend(quote! {pl_deserializer.read_all(#id)?, });
                                } else {
                                    match default_value {
                                    Some(default) => field_deserialization.extend(quote!{pl_deserializer.read_with_default(#id, #default)?, }),
                                    None => field_deserialization.extend(quote!{pl_deserializer.read(#id)?, }),
                                }
                                }
                            } else {
                                let field_name =
                                    field.ident.as_ref().expect("Should have named fields");
                                if read_all_elements {
                                    field_deserialization.extend(
                                        quote! {#field_name: pl_deserializer.read_all(#id)?,},
                                    );
                                } else {
                                    match default_value {
                                    Some(default) => field_deserialization
                                    .extend(quote! {#field_name: pl_deserializer.read_with_default(#id, #default)?,}),
                                    None => field_deserialization
                                    .extend(quote! {#field_name: pl_deserializer.read(#id)?,}),
                                }
                                }
                            }
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
                    impl #generics dust_dds::cdr::parameter_list_deserialize::ParameterListDeserialize<'__de> for #ident #type_generics #where_clause {
                        fn deserialize(pl_deserializer: &mut dust_dds::cdr::parameter_list_deserializer::ParameterListDeserializer<'__de>) -> Result<Self, std::io::Error> {
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
