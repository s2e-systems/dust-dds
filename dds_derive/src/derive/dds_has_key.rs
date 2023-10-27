use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

use crate::attribute_helpers::struct_has_key;

pub fn expand_has_key(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(data_struct) => {
            let has_key = struct_has_key(data_struct);
            let ident = &input.ident;
            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
            Ok(quote!(
                impl #impl_generics dust_dds::topic_definition::type_support::DdsHasKey for #ident #type_generics #where_clause {
                    const HAS_KEY: bool = #has_key;
                }
            ))
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
    fn struct_with_key() {
        let input = syn::parse2::<DeriveInput>(
            "
            struct WithKey {
                #[key]
                id: u8
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let result = syn::parse2::<ItemImpl>(expand_has_key(&input).unwrap()).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            impl dust_dds::topic_definition::type_support::DdsHasKey for WithKey {
                const HAS_KEY: bool = true;
            }"
            .parse()
            .unwrap(),
        )
        .unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn struct_without_key() {
        let input = syn::parse2::<DeriveInput>(
            "
            struct WithoutKey {
                data: u32
            }
        "
            .parse()
            .unwrap(),
        )
        .unwrap();

        let result = syn::parse2::<ItemImpl>(expand_has_key(&input).unwrap()).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            impl dust_dds::topic_definition::type_support::DdsHasKey for WithoutKey {
                const HAS_KEY: bool = false;
            }"
            .parse()
            .unwrap(),
        )
        .unwrap();

        assert_eq!(result, expected);
    }
}
