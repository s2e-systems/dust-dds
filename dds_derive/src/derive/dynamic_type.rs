use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn expand_xtypes_dynamic_type(input: &DeriveInput) -> syn::Result<TokenStream> {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;
    let ident_str = ident.to_string();

    match &input.data {
        Data::Struct(data_struct) => {
            let member_count = data_struct.fields.len() as u32;
            Ok(quote! {
                impl #impl_generics xtypes::dynamic_type::DynamicType for #ident #type_generics #where_clause {
                    fn get_name(&self) -> xtypes::dynamic_type::ObjectName {
                        #ident_str
                    }

                    fn get_kind(&self) -> xtypes::dynamic_type::TypeKind {
                        xtypes::dynamic_type::TK_STRUCTURE
                    }

                    fn get_member_count(&self) -> u32 {
                        #member_count
                    }

                    fn get_member_by_index(&self, index: u32) -> Result<impl DynamicTypeMember, xtypes::error::XcdrError> {
                        match index {
                            _ => Err(xtypes::error::XcdrError::InvalidIndex),
                        }
                    }
                }
            })
        }
        Data::Enum(_) => Ok(quote! {
            impl #impl_generics xtypes::dynamic_type::DynamicType for #ident #type_generics #where_clause {
                fn get_name(&self) -> xtypes::dynamic_type::ObjectName {
                    #ident_str
                }

                fn get_kind(&self) -> xtypes::dynamic_type::TypeKind {
                    xtypes::dynamic_type::TK_ENUM
                }

                fn get_member_count(&self) -> u32 {
                    0
                }
            }
        }),
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
            impl xtypes::dynamic_type::DynamicType for MyData {
                fn get_name(&self) -> xtypes::dynamic_type::ObjectName {
                    \"MyData\"
                }

                fn get_kind(&self) -> xtypes::dynamic_type::TypeKind {
                    xtypes::dynamic_type::TK_STRUCTURE
                }

                fn get_member_count(&self) -> u32 {
                    2u32
                }

                fn get_member_by_index(&self, index: u32) -> Result<impl DynamicTypeMember, xtypes::error::XcdrError> {
                    match index {
                        _ => Err(xtypes::error::XcdrError::InvalidIndex),
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
