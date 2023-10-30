use quote::ToTokens;
use syn::{spanned::Spanned, Attribute, DataStruct, Field};

pub fn field_has_key_attribute(field: &Field) -> bool {
    field.attrs.iter().any(|attr| attr.path().is_ident("key"))
}

pub fn struct_has_key(data_struct: &DataStruct) -> bool {
    data_struct.fields.iter().any(field_has_key_attribute)
}

// Get parameter attributes. Returns (id, Option<default>, serialize_elements)
pub fn get_parameter_attributes(
    parameter_attribute: &Attribute,
) -> syn::Result<(syn::Expr, Option<syn::Expr>, bool)> {
    let mut id: Option<syn::Expr> = None;
    let mut default: Option<syn::Expr> = None;
    let mut serialize_elements = false;
    parameter_attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("id") {
            id = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("default") {
            default = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("serialize_elements") {
            serialize_elements = true;
            Ok(())
        } else {
            Err(syn::Error::new(
                meta.path.span(),
                format!(
                    "Unexpected element {}. Valid options are \"id\", \"default\" and \"serialize_elements\".",
                    meta.path.into_token_stream().to_string(),
                ),
            ))
        }
    })?;

    Ok((
        id.ok_or(syn::Error::new(
            parameter_attribute.span(),
            "\"id\" attribute not found",
        ))?,
        default,
        serialize_elements,
    ))
}
