mod derive;

use derive::{
    cdr::{expand_cdr_deserialize, expand_cdr_serialize},
    dds_key::{expand_dds_key, expand_has_key},
    dds_serialize_data::{expand_dds_deserialize_data, expand_dds_serialize_data},
    dds_type_xml::expand_dds_type_xml,
    dynamic_type::expand_xtypes_dynamic_type,
    parameter_list::{expand_parameter_list_deserialize, expand_parameter_list_serialize},
    xtypes::{expand_xtypes_deserialize, expand_xtypes_serialize},
};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(XTypesDynamicType, attributes(xtypes))]
pub fn derive_xtypes_dynamic_type(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_xtypes_dynamic_type(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(XTypesSerialize, attributes(xtypes))]
pub fn derive_xtypes_serialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_xtypes_serialize(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(XTypesDeserialize, attributes(xtypes))]
pub fn derive_xtypes_deserialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_xtypes_deserialize(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(CdrSerialize)]
pub fn derive_cdr_serialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_cdr_serialize(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(CdrDeserialize)]
pub fn derive_cdr_deserialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_cdr_deserialize(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(ParameterListSerialize, attributes(parameter))]
pub fn derive_parameter_list_serialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_parameter_list_serialize(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(ParameterListDeserialize, attributes(parameter))]
pub fn derive_parameter_list_deserialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_parameter_list_deserialize(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DdsSerialize, attributes(dust_dds))]
pub fn derive_dds_serialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_dds_serialize_data(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DdsDeserialize, attributes(dust_dds))]
pub fn derive_dds_deserialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_dds_deserialize_data(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DdsHasKey, attributes(dust_dds))]
pub fn derive_dds_has_key(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_has_key(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DdsKey, attributes(dust_dds))]
pub fn derive_dds_key(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_dds_key(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DdsTypeXml, attributes(dust_dds))]
pub fn derive_dds_type_xml(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_dds_type_xml(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DdsType, attributes(dust_dds))]
pub fn derive_dds_type(input: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();

    output.extend(derive_cdr_serialize(input.clone()));
    output.extend(derive_cdr_deserialize(input.clone()));
    output.extend(derive_dds_serialize(input.clone()));
    output.extend(derive_dds_deserialize(input.clone()));
    output.extend(derive_dds_key(input.clone()));
    output.extend(derive_dds_has_key(input.clone()));
    output.extend(derive_dds_type_xml(input));

    output
}
