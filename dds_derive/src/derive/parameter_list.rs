use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Attribute, DeriveInput, Index, Result};

fn get_parameter_attributes(
    parameter_attribute: &Attribute,
) -> syn::Result<(syn::Expr, Option<syn::Expr>, bool, bool)> {
    let mut id: Option<syn::Expr> = None;
    let mut default: Option<syn::Expr> = None;
    let mut collection = false;
    let mut skip_serialize = false;
    parameter_attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("id") {
            id = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("default") {
            default = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("collection") {
            collection = true;
            Ok(())
        } else if meta.path.is_ident("skip_serialize") {
            skip_serialize = true;
            Ok(())
        } else {
            Err(syn::Error::new(
                meta.path.span(),
                format!(
                    r#"Unexpected element {}. Valid options are "id", "default", "collection", "skip_serialize"."#,
                    meta.path.into_token_stream(),
                ),
            ))
        }
    })?;

    Ok((
        id.ok_or_else(|| {
            syn::Error::new(parameter_attribute.span(), "\"id\" attribute not found")
        })?,
        default,
        collection,
        skip_serialize,
    ))
}
