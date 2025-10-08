use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Result};

pub fn expand_dds_deserialize_data(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(_) | syn::Data::Enum(_) => {
            let (_, type_generics, where_clause) = input.generics.split_for_impl();

            // Create a '__de lifetime bound to all the lifetimes of the struct
            let mut de_lifetime_param =
                syn::LifetimeParam::new(syn::Lifetime::new("'__de", Span::call_site()));
            for struct_lifetime in input.generics.lifetimes().cloned() {
                de_lifetime_param.bounds.push(struct_lifetime.lifetime);
            }

            // Append the '__de lifetime to the impl generics of the struct
            let mut generics = input.generics.clone();
            generics.params = Some(syn::GenericParam::Lifetime(de_lifetime_param))
                .into_iter()
                .chain(generics.params)
                .collect();

            let ident = &input.ident;

            let deserialize_function = quote! {
                dust_dds::infrastructure::type_support::deserialize_rtps_encapsulated_data(&mut serialized_data)
            };

            Ok(quote! {
                #[automatically_derived]
                impl #generics dust_dds::infrastructure::type_support::DdsDeserialize<'__de> for #ident #type_generics #where_clause {
                    fn deserialize_data(mut serialized_data: &'__de [u8]) -> dust_dds::infrastructure::error::DdsResult<Self> {
                        #deserialize_function
                    }
                }
            })
        }
        syn::Data::Union(data_union) => Err(syn::Error::new(
            data_union.union_token.span,
            "Union not supported",
        )),
    }
}
