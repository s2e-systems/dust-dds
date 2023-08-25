use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Field};

#[proc_macro_derive(DdsType, attributes(key))]
pub fn derive_dds_type(input: TokenStream) -> TokenStream {
    fn field_has_key_attribute(field: &Field) -> bool {
        field.attrs.iter().any(|attr| {
            attr.parse_meta()
                .ok()
                .and_then(|meta| meta.path().get_ident().cloned())
                .map(|ident| ident == "key")
                .unwrap_or(false)
        })
    }

    let input: DeriveInput = parse_macro_input!(input);

    if let syn::Data::Struct(struct_data) = &input.data {
        let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
        let ident = input.ident;

        let has_key = struct_data.fields.iter().any(field_has_key_attribute);

        quote! {
            impl #impl_generics dust_dds::topic_definition::type_support::DdsType for #ident #type_generics #where_clause {
                const REPRESENTATION_IDENTIFIER: dust_dds::topic_definition::type_support::RepresentationType
                    = dust_dds::topic_definition::type_support::CDR_LE;

                fn has_key() -> bool {
                    #has_key
                }

            }
        }
    } else {
        quote_spanned!{input.span() => compile_error!("DdsType can only be derived for structs");}
    }
    .into()
}
