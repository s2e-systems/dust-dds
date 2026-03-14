use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Expr, Field, Ident,
    parse::{Parse, ParseStream},
};

pub struct FieldAttributes {
    pub key: Option<Key>,
    pub optional: bool,
    pub id: Option<Expr>,
    pub default_value: Option<Expr>,
    pub non_serialized: bool,
}

pub fn get_field_attributes(field: &Field) -> syn::Result<FieldAttributes> {
    let mut key = None;
    let mut optional = false;
    let mut id = None;
    let mut default_value = None;
    let mut non_serialized = false;
    if let Some(xtypes_attribute) = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("dust_dds"))
    {
        xtypes_attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("key") {
                key = Some(meta.input.parse()?);
            } else if meta.path.is_ident("id") {
                id = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("default_value") {
                default_value = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("optional") {
                optional = true;
            } else if meta.path.is_ident("non_serialized") {
                non_serialized = true;
            }
            Ok(())
        })?;
    }
    Ok(FieldAttributes {
        key,
        optional,
        id,
        default_value,
        non_serialized,
    })
}

pub enum Key {
    Key,
    Transparent,
}

impl Parse for Key {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::Key);
        }

        let content;
        syn::parenthesized!(content in input);
        let ident: Ident = content.parse()?;

        if ident != "transparent" {
            return Err(syn::Error::new(
                ident.span(),
                format!("`{ident}` is invalid, expected `transparent`"),
            ));
        }

        Ok(Self::Transparent)
    }
}

impl ToTokens for Key {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Key => quote! { ::dust_dds::xtypes::dynamic_type::MemberKey::Key },
            Self::Transparent => {
                quote! { ::dust_dds::xtypes::dynamic_type::MemberKey::Transparent }
            }
        }
        .to_tokens(tokens);
    }
}
