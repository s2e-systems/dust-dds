use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Index, Result};

use crate::attribute_helpers::{
    get_parameter_default_attribute, get_parameter_id_attribute, get_parameter_serialize_elements,
};

pub fn expand_parameter_list_serialize(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(data_struct) => {
            let mut field_serialization = quote! {};
            let ident = &input.ident;
            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

            for (field_index, field) in data_struct.fields.iter().enumerate() {
                let id = get_parameter_id_attribute(field)?;
                let serialize_elements = get_parameter_serialize_elements(field)?;

                if !serialize_elements {
                    let default_value = get_parameter_default_attribute(field)?;
                    match (&field.ident, default_value) {
                        (Some(field_name), None) => field_serialization.extend(quote! {
                            serializer.write(#id, &self.#field_name)?;
                        }),
                        (Some(field_name), Some(default)) => field_serialization.extend(quote! {
                            serializer.write_with_default(#id, &self.#field_name, #default)?;
                        }),
                        (None, None) => {
                            let index = Index::from(field_index);
                            field_serialization.extend(quote! {
                                serializer.write(#id, &self.#index)?;
                            })
                        }
                        (None, Some(default)) => {
                            let index = Index::from(field_index);
                            field_serialization.extend(quote! {
                                serializer.write_with_default(#id, &self.#index, #default)?;
                            })
                        }
                    }
                } else {
                    match &field.ident {
                        Some(field_name) => field_serialization.extend(quote! {
                            serializer.write_list_elements(#id, &self.#field_name)?;
                        }),
                        None => {
                            let index = Index::from(field_index);
                            field_serialization.extend(quote! {
                                serializer.write_list_elements(#id, &self.#index)?;
                            })
                        }
                    }
                }
            }

            Ok(quote! {
                impl #impl_generics dust_dds::cdr::parameter_list_serialize::ParameterListSerialize for #ident #type_generics #where_clause {
                    fn serialize(&self, serializer: &mut dust_dds::cdr::parameter_list_serializer::ParameterListSerializer) -> Result<(), std::io::Error> {
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
            impl dust_dds::cdr::parameter_list_serialize::ParameterListSerialize for ParameterListStruct {
                fn serialize(&self, serializer: &mut dust_dds::cdr::parameter_list_serializer::ParameterListSerializer) -> Result<(), std::io::Error> {
                    serializer.write(1, &self.index)?;
                    serializer.write(PID_DATA, &self.data)?;
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
            impl dust_dds::cdr::parameter_list_serialize::ParameterListSerialize for ParameterListStructDefault {
                fn serialize(&self, serializer: &mut dust_dds::cdr::parameter_list_serializer::ParameterListSerializer) -> Result<(), std::io::Error> {
                    serializer.write(1, &self.index)?;
                    serializer.write(PID_DATA, &self.data)?;
                    serializer.write_with_default(3, &self.name, \"\")?;
                    serializer.write_with_default(4, &self.x, Default::default())?;
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
