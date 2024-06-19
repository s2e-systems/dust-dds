use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, Result};

enum Format {
    CdrLe,
    CdrBe,
    PlCdrLe,
    PlCdrBe,
}

fn get_format(input: &DeriveInput) -> Result<Format> {
    let mut format = Format::CdrLe;
    if let Some(dds_attribute) = input.attrs.iter().find(|&a| a.path().is_ident("dust_dds")) {
        dds_attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("format") {
                let format_str: syn::LitStr = meta.value()?.parse()?;
                match format_str.value().as_ref() {
                    "CDR_LE" => {
                        format = Format::CdrLe;
                        Ok(())
                    }
                    "CDR_BE" => {
                        format = Format::CdrBe;
                        Ok(())
                    }
                    "PL_CDR_LE" => {
                        format = Format::PlCdrLe;
                        Ok(())
                    }
                    "PL_CDR_BE" => {
                        format = Format::PlCdrBe;
                        Ok(())
                    }
                    _ => Err(syn::Error::new(
                        meta.path.span(),
                        r#"Invalid format specified. Valid options are "CDR_LE", "CDR_BE", "PL_CDR_LE", "PL_CDR_BE". "#,
                    )),
                }
            } else {
                Ok(())
            }
        })?;
    }
    Ok(format)
}

pub fn expand_dds_serialize_data(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(_) | syn::Data::Enum(_) => {
            let format = get_format(input)?;
            let serialize_function = match format {
                Format::CdrLe => quote! {
                    dust_dds::topic_definition::type_support::serialize_rtps_classic_cdr_le(
                        self,
                )},
                Format::CdrBe => quote! {
                    dust_dds::topic_definition::type_support::serialize_rtps_classic_cdr_be(
                        self,
                )},
                Format::PlCdrLe => quote! {
                    dust_dds::topic_definition::type_support::serialize_rtps_cdr_pl_le(
                        self,
                )},
                Format::PlCdrBe => quote! {
                    dust_dds::topic_definition::type_support::serialize_rtps_cdr_pl_be(
                        self,
                )},
            };

            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
            let ident = &input.ident;

            Ok(quote! {
                impl #impl_generics dust_dds::topic_definition::type_support::DdsSerialize for #ident #type_generics #where_clause {
                    fn serialize_data(&self) -> dust_dds::infrastructure::error::DdsResult<Vec<u8>> {
                        #serialize_function
                    }
                }
            })
        },
        syn::Data::Union(data_union) => Err(syn::Error::new(
            data_union.union_token.span,
            "Union not supported",
        )),
    }
}

pub fn expand_dds_deserialize_data(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(_) | syn::Data::Enum(_)=> {
            let format = get_format(input)?;
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

            let deserialize_function = match format {
                Format::CdrLe | Format::CdrBe => {
                    quote! {
                        dust_dds::topic_definition::type_support::deserialize_rtps_classic_cdr(&mut serialized_data)
                    }
                }
                Format::PlCdrLe | Format::PlCdrBe => quote! {
                    dust_dds::topic_definition::type_support::deserialize_rtps_cdr_pl(&mut serialized_data)
                },
            };

            Ok(quote! {
                impl #generics dust_dds::topic_definition::type_support::DdsDeserialize<'__de> for #ident #type_generics #where_clause {
                    fn deserialize_data(mut serialized_data: &'__de [u8]) -> dust_dds::infrastructure::error::DdsResult<Self> {
                        #deserialize_function
                    }
                }
            })
        },
        syn::Data::Union(data_union) => Err(syn::Error::new(
            data_union.union_token.span,
            "Union not supported",
        )),
    }
}
