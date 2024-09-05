mod derive;

use derive::{
    dynamic_type::expand_xtypes_dynamic_type,
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
