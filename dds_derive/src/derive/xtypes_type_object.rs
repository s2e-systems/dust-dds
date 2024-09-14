use super::{
    attributes::{get_field_id, get_input_extensibility, Extensibility},
    enum_support::{
        get_enum_bitbound, is_enum_xtypes_union, read_enum_variant_discriminant_mapping, BitBound,
    },
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Fields, Index, Result};

pub fn expand_xtypes_type_object(input: &DeriveInput) -> Result<TokenStream> {
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;

    let type_object_quote = match &input.data {
        syn::Data::Struct(data_struct) => Ok(quote! {}),
        syn::Data::Enum(data_enum) => Ok(quote! {}),
        syn::Data::Union(data_union) => Err(syn::Error::new(
            data_union.union_token.span,
            "Union not supported",
        )),
    }?;

    Ok(quote! {
        impl #impl_generics  dust_dds::xtypes::type_object::XTypesTypeObject for #ident #type_generics #where_clause {
            fn type_object() -> dust_dds::xtypes::type_object::TypeObject
            {
                #type_object_quote
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;
    use syn::ItemImpl;

    use super::*;

    #[test]
    fn xtypes_serialize_final_struct_with_basic_types() {
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

        let output_token_stream = expand_xtypes_type_object(&input).unwrap();
        let result = syn::parse2::<ItemImpl>(output_token_stream).unwrap();
        let expected = syn::parse2::<ItemImpl>(
            "
            impl  dust_dds::xtypes::type_object::XTypesTypeObject for MyData {
                fn type_object() -> dust_dds::xtypes::type_object::TypeObject {

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
