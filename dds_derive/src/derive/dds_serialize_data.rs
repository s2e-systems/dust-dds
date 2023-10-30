use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

pub fn expand_dds_serialize_data(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(data_struct) => {
            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
            let ident = &input.ident;

            Ok(quote! {
                impl #impl_generics dust_dds::topic_definition::type_support::DdsSerializeData for #ident #type_generics #where_clause {
                    fn serialize_data(&self) -> dust_dds::infrastructure::error::DdsResult<dust_dds::topic_definition::type_support::DdsSerializedData> {
                        todo!()
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
