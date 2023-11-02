use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Index, Result};

pub fn expand_cdr_serialize(input: &DeriveInput) -> Result<TokenStream> {
    let mut field_serialization = quote!();

    match &input.data {
        syn::Data::Struct(data_struct) => {
            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
            let ident = &input.ident;

            for (field_index, field) in data_struct.fields.iter().enumerate() {
                match &field.ident {
                    Some(field_name) => {
                        field_serialization
                            .extend(quote! {dust_dds::serialized_payload::cdr::serialize::CdrSerialize::serialize(&self.#field_name, serializer)?;});
                    }
                    None => {
                        let index = Index::from(field_index);
                        field_serialization.extend(quote! {dust_dds::serialized_payload::cdr::serialize::CdrSerialize::serialize(&self.#index, serializer)?;});
                    }
                }
            }

            Ok(quote! {
                impl #impl_generics dust_dds::serialized_payload::cdr::serialize::CdrSerialize for #ident #type_generics #where_clause {
                    fn serialize(&self, serializer: &mut impl dust_dds::serialized_payload::cdr::serializer::CdrSerializer) -> Result<(), std::io::Error> {
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
