mod derive;
use derive::{
    dds_serialize_data::{expand_dds_deserialize_data, expand_dds_serialize_data},
    type_support::expand_type_support,
    xtypes::{expand_xtypes_deserialize, expand_xtypes_serialize},
};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(XTypesSerialize, attributes(dust_dds))]
pub fn derive_xtypes_serialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_xtypes_serialize(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(XTypesDeserialize, attributes(dust_dds))]
pub fn derive_xtypes_deserialize(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_xtypes_deserialize(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(TypeSupport, attributes(dust_dds))]
pub fn derive_type_support(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_type_support(&input)
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

#[proc_macro_derive(DdsType, attributes(dust_dds))]
pub fn derive_dds_type(input: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();

    output.extend(derive_xtypes_serialize(input.clone()));
    output.extend(derive_xtypes_deserialize(input.clone()));
    output.extend(derive_dds_serialize(input.clone()));
    output.extend(derive_dds_deserialize(input.clone()));
    output.extend(derive_type_support(input));

    output
}
