use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

use super::attributes::{field_has_key_attribute, get_input_extensibility, Extensibility};

pub fn expand_xtypes_dynamic_type(input: &DeriveInput) -> syn::Result<TokenStream> {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;
    let ident_str = ident.to_string();

    match &input.data {
        Data::Struct(data_struct) => {
            let member_count = data_struct.fields.len() as u32;
            let extensibility = get_input_extensibility(input)?;
            let extinsibility_kind = match extensibility {
                Extensibility::Final => quote! { dust_dds::dust_dds_xtypes::dynamic_type::ExtensibilityKind::Final},
                Extensibility::Appendable => {
                    quote! { dust_dds::dust_dds_xtypes::dynamic_type::ExtensibilityKind::Appendable}
                }
                Extensibility::Mutable => quote! { dust_dds::dust_dds_xtypes::dynamic_type::ExtensibilityKind::Mutable},
            };
            let mut dynamic_type_member = quote! {};
            for (field_index, field) in data_struct.fields.iter().enumerate() {
                let field_index = field_index as u32;
                let name = match &field.ident {
                    Some(i) => i.to_string(),
                    None => String::new(),
                };
                let is_key = field_has_key_attribute(field)?;
                dynamic_type_member.extend(quote! {#field_index => Ok(
                     dust_dds::dust_dds_xtypes::dynamic_type::MemberDescriptor {
                        name: #name,
                        id: 0,
                        default_value: "",
                        index: #field_index,
                        is_key: #is_key,
                        is_optional: false,
                        is_must_understand: true,
                        is_shared: false,
                        is_default_label: false,
                    }
                ),})
            }
            Ok(quote! {
                impl #impl_generics  dust_dds::dust_dds_xtypes::dynamic_type::DynamicType for #ident #type_generics #where_clause {
                    fn get_descriptor(&self) -> Result< dust_dds::dust_dds_xtypes::dynamic_type::TypeDescriptor,  dust_dds::dust_dds_xtypes::error::XcdrError> {
                        Ok( dust_dds::dust_dds_xtypes::dynamic_type::TypeDescriptor {
                            kind:  dust_dds::dust_dds_xtypes::dynamic_type::TK_STRUCTURE,
                            name: #ident_str,
                            extensibility_kind: #extinsibility_kind,
                            is_nested: false,
                        })
                    }

                    fn get_name(&self) ->  dust_dds::dust_dds_xtypes::dynamic_type::ObjectName {
                        #ident_str
                    }

                    fn get_kind(&self) ->  dust_dds::dust_dds_xtypes::dynamic_type::TypeKind {
                         dust_dds::dust_dds_xtypes::dynamic_type::TK_STRUCTURE
                    }

                    fn get_member_count(&self) -> u32 {
                        #member_count
                    }

                    fn get_member_by_index(&self, index: u32) -> Result<impl  dust_dds::dust_dds_xtypes::dynamic_type::DynamicTypeMember,  dust_dds::dust_dds_xtypes::error::XcdrError> {
                        match index {
                            #dynamic_type_member
                            _ => Err( dust_dds::dust_dds_xtypes::error::XcdrError::InvalidIndex),
                        }
                    }
                }
            })
        }
        Data::Enum(data_enum) => {
            let enum_variant_count = data_enum.variants.len();
            let extensibility = get_input_extensibility(input)?;
            let extinsibility_kind = match extensibility {
                Extensibility::Final => quote! { dust_dds::dust_dds_xtypes::dynamic_type::ExtensibilityKind::Final},
                Extensibility::Appendable => {
                    quote! { dust_dds::dust_dds_xtypes::dynamic_type::ExtensibilityKind::Appendable}
                }
                Extensibility::Mutable => {
                    panic!("Mutable extensibility kind not allowed for enumeration types")
                }
            };
            Ok(quote! {
                impl #impl_generics  dust_dds::dust_dds_xtypes::dynamic_type::DynamicType for #ident #type_generics #where_clause {
                    fn get_descriptor(&self) -> Result< dust_dds::dust_dds_xtypes::dynamic_type::TypeDescriptor,  dust_dds::dust_dds_xtypes::error::XcdrError> {
                        Ok( dust_dds::dust_dds_xtypes::dynamic_type::TypeDescriptor {
                            kind:  dust_dds::dust_dds_xtypes::dynamic_type::TK_ENUM,
                            name: #ident_str,
                            extensibility_kind: #extinsibility_kind,
                            is_nested: false,
                        })
                    }

                    fn get_name(&self) ->  dust_dds::dust_dds_xtypes::dynamic_type::ObjectName {
                        #ident_str
                    }

                    fn get_kind(&self) ->  dust_dds::dust_dds_xtypes::dynamic_type::TypeKind {
                         dust_dds::dust_dds_xtypes::dynamic_type::TK_ENUM
                    }

                    fn get_member_count(&self) -> u32 {
                        #enum_variant_count
                    }

                    fn get_member_by_index(&self, index: u32) -> Result<impl  dust_dds::dust_dds_xtypes::dynamic_type::DynamicTypeMember,  dust_dds::dust_dds_xtypes::error::XcdrError> {
                        match index {
                            _ => Err( dust_dds::dust_dds_xtypes::error::XcdrError::InvalidIndex),
                        }
                    }
                }
            })
        }
        Data::Union(data_union) => Err(syn::Error::new(
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
    fn xtypes_dynamic_type_struct_with_basic_types() {
        let input = syn::parse2::<DeriveInput>(
            "
            #[xtypes(extensibility = \"Final\")]
            struct MyData {
                #[xtypes(key)]
                x: u32,
                y: u32,
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let output_token_stream = expand_xtypes_dynamic_type(&input).unwrap();
        let result = syn::parse2::<ItemImpl>(output_token_stream).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            impl  dust_dds::dust_dds_xtypes::dynamic_type::DynamicType for MyData {
                fn get_descriptor(&self) -> Result< dust_dds::dust_dds_xtypes::dynamic_type::TypeDescriptor,  dust_dds::dust_dds_xtypes::error::XcdrError> {
                    Ok( dust_dds::dust_dds_xtypes::dynamic_type::TypeDescriptor {
                        kind:  dust_dds::dust_dds_xtypes::dynamic_type::TK_STRUCTURE,
                        name: \"MyData\",
                        extensibility_kind:  dust_dds::dust_dds_xtypes::dynamic_type::ExtensibilityKind::Final,
                        is_nested: false,
                    })
                }

                fn get_name(&self) ->  dust_dds::dust_dds_xtypes::dynamic_type::ObjectName {
                    \"MyData\"
                }

                fn get_kind(&self) ->  dust_dds::dust_dds_xtypes::dynamic_type::TypeKind {
                     dust_dds::dust_dds_xtypes::dynamic_type::TK_STRUCTURE
                }

                fn get_member_count(&self) -> u32 {
                    2u32
                }

                fn get_member_by_index(&self, index: u32) -> Result<impl  dust_dds::dust_dds_xtypes::dynamic_type::DynamicTypeMember,  dust_dds::dust_dds_xtypes::error::XcdrError> {
                    match index {
                        0u32 => Ok(
                             dust_dds::dust_dds_xtypes::dynamic_type::MemberDescriptor {
                                name: \"x\",
                                id: 0,
                                default_value: \"\",
                                index: 0u32,
                                is_key: true,
                                is_optional: false,
                                is_must_understand: true,
                                is_shared: false,
                                is_default_label: false,
                            }
                        ),
                        1u32 => Ok(
                             dust_dds::dust_dds_xtypes::dynamic_type::MemberDescriptor {
                                name: \"y\",
                                id: 0,
                                default_value: \"\",
                                index: 1u32,
                                is_key: false,
                                is_optional: false,
                                is_must_understand: true,
                                is_shared: false,
                                is_default_label: false,
                            }
                        ),
                        _ => Err( dust_dds::dust_dds_xtypes::error::XcdrError::InvalidIndex),
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
