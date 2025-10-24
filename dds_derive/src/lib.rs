mod derive;
use derive::type_support::expand_type_support;
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(TypeSupport, attributes(dust_dds))]
pub fn derive_type_support(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    expand_type_support(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DdsType, attributes(dust_dds))]
pub fn derive_dds_type(input: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();

    output.extend(derive_type_support(input));

    output
}
