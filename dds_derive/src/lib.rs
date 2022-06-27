use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(DdsType, attributes(dds_type_with_key))]
pub fn derive_dds_type(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, attrs, .. } = parse_macro_input!(input);
    let type_name = ident.to_string();
    let has_key = attrs.iter().any(|attr| {
        attr.path.get_ident().map(|ident| ident.to_string())
            == Some("dds_type_with_key".to_string())
    });

    quote! {
        impl DdsType for #ident {
            fn type_name() -> &'static str {
                #type_name
            }

            fn has_key() -> bool {
                #has_key
            }
        }
    }
    .into()
}