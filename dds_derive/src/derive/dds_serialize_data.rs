use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, LitStr, Result};

pub fn expand_dds_serialize_data(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        syn::Data::Struct(data_struct) => {
            enum Format {
                CdrLe,
                CdrBe,
                PlCdrLe,
                PlCdrBe,
            }

            let mut format = Format::CdrLe;
            if let Some(dds_attribute) = input.attrs.iter().find(|&a| a.path().is_ident("dust_dds"))
            {
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

            let serialize_function = match format {
                Format::CdrLe => quote! {
                    dust_dds::cdr::representation::serialize_rtps_cdr(
                        self,
                        writer,
                        dust_dds::cdr::endianness::CdrEndianness::LittleEndian,
                )?;},
                Format::CdrBe => quote! {
                    dust_dds::cdr::representation::serialize_rtps_cdr(
                        self,
                        writer,
                        dust_dds::cdr::endianness::CdrEndianness::BigEndian,
                )?;},
                Format::PlCdrLe => quote! {
                    dust_dds::cdr::representation::serialize_rtps_cdr_pl(
                        self,
                        writer,
                        dust_dds::cdr::endianness::CdrEndianness::LittleEndian,
                )?;},
                Format::PlCdrBe => quote! {
                    dust_dds::cdr::representation::serialize_rtps_cdr_pl(
                        self,
                        writer,
                        dust_dds::cdr::endianness::CdrEndianness::BigEndian,
                )?;},
            };

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
