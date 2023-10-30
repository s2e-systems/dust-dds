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
        syn::Data::Struct(_data_struct) => {
            let format = get_format(input)?;
            let serialize_function = match format {
                Format::CdrLe => quote! {
                    dust_dds::topic_definition::type_support::serialize_rtps_cdr(
                        self,
                        writer,
                        dust_dds::cdr::endianness::CdrEndianness::LittleEndian,
                )},
                Format::CdrBe => quote! {
                    dust_dds::topic_definition::type_support::serialize_rtps_cdr(
                        self,
                        writer,
                        dust_dds::cdr::endianness::CdrEndianness::BigEndian,
                )},
                Format::PlCdrLe => quote! {
                    dust_dds::topic_definition::type_support::serialize_rtps_cdr_pl(
                        self,
                        writer,
                        dust_dds::cdr::endianness::CdrEndianness::LittleEndian,
                )},
                Format::PlCdrBe => quote! {
                    dust_dds::topic_definition::type_support::serialize_rtps_cdr_pl(
                        self,
                        writer,
                        dust_dds::cdr::endianness::CdrEndianness::BigEndian,
                )},
            };

            let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
            let ident = &input.ident;

            Ok(quote! {
                impl #impl_generics dust_dds::topic_definition::type_support::DdsSerialize for #ident #type_generics #where_clause {
                    fn serialize_data(&self, writer: &mut Vec<u8>) -> dust_dds::infrastructure::error::DdsResult<()> {
                        #serialize_function
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

pub fn expand_dds_deserialize_data(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(_data_struct) => {
            let format = get_format(input)?;

            let (_, type_generics, where_clause) = input.generics.split_for_impl();

            // Append the '__de lifetime to the impl generics of the struct
            let mut generics = input.generics.clone();
            generics.params = Some(syn::GenericParam::Lifetime(syn::LifetimeParam::new(
                syn::Lifetime::new("'__de", Span::call_site()),
            )))
            .into_iter()
            .chain(generics.params)
            .collect();

            let ident = &input.ident;

            Ok(quote! {
                impl #generics dust_dds::topic_definition::type_support::DdsDeserialize<'__de> for #ident #type_generics #where_clause {
                    fn deserialize_data(serialized_data: &'__de [u8]) -> dust_dds::infrastructure::error::DdsResult<Self> {
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
