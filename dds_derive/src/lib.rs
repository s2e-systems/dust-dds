use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, DataEnum, DataStruct, DataUnion, DeriveInput};

#[proc_macro_derive(DdsType, attributes(key))]
pub fn derive_dds_type(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let ident = input.ident;
    let type_name = ident.to_string();

    let key = match input.data {
        syn::Data::Struct(struct_data) => struct_key(&input.attrs, struct_data),
        syn::Data::Enum(enum_data) => enum_key(&input.attrs, enum_data),
        syn::Data::Union(union_data) => union_key(&input.attrs, union_data),
    };

    quote! {
        impl #impl_generics DdsType for #ident #type_generics #where_clause {
            fn type_name() -> &'static str {
                #type_name
            }

            #key
        }
    }
    .into()
}

fn struct_key(input_attrs: &[Attribute], struct_data: DataStruct) -> TokenStream2 {
    let mut errors = TokenStream2::new();

    let is_key = find_key_attribute(input_attrs).is_some();

    let mut build_key = quote! {
        let mut serialised_key = Vec::new();
    };
    let mut has_key = false;
    for (field_number, field) in struct_data.fields.iter().enumerate() {
        if let Some(attr) = find_key_attribute(&field.attrs) {
            has_key = true;

            if is_key {
                errors.extend(quote_spanned!{
                    attr.path.span() => compile_error!(
                        "Using #[key] on fields is undefined when the whole struct is already marked with #[key]"
                    );
                });
            } else {
                let field_ident = field
                    .ident
                    .clone()
                    .map(|field| field.into_token_stream())
                    .unwrap_or(syn::Index::from(field_number).into_token_stream());

                build_key.extend(quote! {
                    serialised_key.extend(
                        if E::REPRESENTATION_IDENTIFIER == dds_implementation::dds_type::PL_CDR_BE {
                            cdr::serialize::<_, _, cdr::CdrBe>(&self.#field_ident, cdr::Infinite).unwrap()
                        } else {
                            cdr::serialize::<_, _, cdr::CdrLe>(&self.#field_ident, cdr::Infinite).unwrap()
                        }
                    );
                })
            }
        }
    }

    build_key.extend(quote! {
        return serialised_key;
    });

    if is_key {
        quote! {
            #errors

            fn has_key() -> bool { true }

            fn serialized_key<E: dds_implementation::dds_type::Endianness>(&self) -> Vec<u8> {
                if E::REPRESENTATION_IDENTIFIER == dds_implementation::dds_type::PL_CDR_BE {
                    cdr::serialize::<_, _, cdr::CdrBe>(self, cdr::Infinite).unwrap()
                } else {
                    cdr::serialize::<_, _, cdr::CdrLe>(self, cdr::Infinite).unwrap()
                }
            }
        }
    } else if has_key {
        quote! {
            #errors

            fn has_key() -> bool { true }

            fn serialized_key<E: dds_implementation::dds_type::Endianness>(&self) -> Vec<u8> {
                #build_key
            }
        }
    } else {
        quote! {
            #errors

            fn has_key() -> bool { false }

            fn serialized_key<E: dds_implementation::dds_type::Endianness>(&self) -> Vec<u8> {
                vec![]
            }
        }
    }
}

fn enum_key(input_attrs: &[Attribute], enum_data: DataEnum) -> TokenStream2 {
    let mut errors = TokenStream2::new();
    let is_key = find_key_attribute(input_attrs).is_some();

    for variant in enum_data.variants {
        if let Some(attr) = find_key_attribute(&variant.attrs) {
            errors.extend(quote_spanned! {
                attr.path.span() => compile_error!("An enum variant cannot be a key");
            });
        }
    }

    if is_key {
        quote! {
            #errors

            fn has_key() -> bool { true }

            fn serialized_key<E: dds_implementation::dds_type::Endianness>(&self) -> Vec<u8> {
                if E::REPRESENTATION_IDENTIFIER == dds_implementation::dds_type::PL_CDR_BE {
                    cdr::serialize::<_, _, cdr::CdrBe>(self, cdr::Infinite).unwrap()
                } else {
                    cdr::serialize::<_, _, cdr::CdrLe>(self, cdr::Infinite).unwrap()
                }
            }
        }
    } else {
        quote! {
            #errors

            fn has_key() -> bool { false }

            fn serialized_key<E: dds_implementation::dds_type::Endianness>(&self) -> Vec<u8> {
                vec![]
            }
        }
    }
}

fn union_key(input_attrs: &[Attribute], union_data: DataUnion) -> TokenStream2 {
    let mut errors = TokenStream2::new();
    let is_key = find_key_attribute(input_attrs).is_some();

    for field in union_data.fields.named.iter() {
        if let Some(attr) = find_key_attribute(&field.attrs) {
            errors.extend(quote_spanned! {
                attr.path.span() => compile_error!("A union variant cannot be a key");
            });
        }
    }

    if is_key {
        quote! {
            #errors

            fn has_key() -> bool { true }

            fn serialized_key<E: dds_implementation::dds_type::Endianness>(&self) -> Vec<u8> {
                if E::REPRESENTATION_IDENTIFIER == dds_implementation::dds_type::PL_CDR_BE {
                    cdr::serialize::<_, _, cdr::CdrBe>(self, cdr::Infinite).unwrap()
                } else {
                    cdr::serialize::<_, _, cdr::CdrLe>(self, cdr::Infinite).unwrap()
                }
            }
        }
    } else {
        quote! {
            #errors

            fn has_key() -> bool { false }

            fn serialized_key<E: dds_implementation::dds_type::Endianness>(&self) -> Vec<u8> {
                vec![]
            }
        }
    }
}

fn find_key_attribute<'a>(attr_list: &'a [Attribute]) -> Option<&'a Attribute> {
    attr_list.iter().find(|attr| {
        attr.parse_meta()
            .ok()
            .and_then(|meta| meta.path().get_ident().cloned())
            .map(|ident| ident.to_string() == "key")
            .unwrap_or(false)
    })
}
