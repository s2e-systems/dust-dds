use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

pub fn expand_dds_serialize_data(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(data_struct) => {
            let dds_attribute = input.attrs.iter().find(|&a| a.path().is_ident("dust_dds"));

            let serialize_function = quote! {
                dust_dds::cdr::representation::serialize_rtps_cdr(
                    self,
                    writer,
                    CdrEndianness::LittleEndian
            )?;};

            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
            let ident = &input.ident;

            Ok(quote! {
                impl #impl_generics dust_dds::topic_definition::type_support::DdsSerializeData for #ident #type_generics #where_clause {
                    fn serialize_data(&self, writer: impl std::io::Write) -> dust_dds::infrastructure::error::DdsResult<()> {
                        #serialize_function
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
