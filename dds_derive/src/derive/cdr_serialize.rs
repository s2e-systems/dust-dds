use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Index, Result};

pub fn expand_cdr_serialize(input: &DeriveInput) -> Result<TokenStream> {
    let mut field_serialization = quote!();

    match &input.data {
        syn::Data::Struct(data_struct) => {
            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
            let ident = &input.ident;

            let mut tuple_field_counter = 0;
            for field in data_struct.fields.iter() {
                match &field.ident {
                    Some(field_name) => {
                        field_serialization
                            .extend(quote! {dust_dds::cdr::serialize::CdrSerialize::serialize(&self.#field_name, serializer)?;});
                    }
                    None => {
                        let index = Index::from(tuple_field_counter);
                        field_serialization.extend(quote! {dust_dds::cdr::serialize::CdrSerialize::serialize(&self.#index, serializer)?;});
                        tuple_field_counter += 1;
                    }
                }
            }

            Ok(quote! {
                const _ : () = {
                    impl #impl_generics dust_dds::cdr::serialize::CdrSerialize for #ident #type_generics #where_clause {
                        fn serialize(&self, serializer: &mut dust_dds::cdr::serializer::CdrSerializer) -> dust_dds::cdr::error::CdrResult<()> {
                            #field_serialization
                            Ok(())
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
