use syn::{spanned::Spanned, DeriveInput, Expr, Field, Result};

pub enum Extensibility {
    Final,
    Appendable,
    Mutable,
}

pub fn get_input_extensibility(input: &DeriveInput) -> Result<Extensibility> {
    let mut extensibility = Extensibility::Final;
    if let Some(xtypes_attribute) = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("dust_dds"))
    {
        xtypes_attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("extensibility") {
                let format_str: syn::LitStr = meta.value()?.parse()?;
                match format_str.value().as_ref() {
                    "Final" => {
                        extensibility = Extensibility::Final;
                        Ok(())
                    }
                    "Appendable" => {
                        extensibility = Extensibility::Appendable;
                        Ok(())
                    }
                    "Mutable" => {
                        extensibility = Extensibility::Mutable;
                        Ok(())
                    }
                    _ => Err(syn::Error::new(
                        meta.path.span(),
                        r#"Invalid format specified. Valid options are "Final", "Appendable", "Mutable". "#,
                    )),
                }
            } else {
                Ok(())
            }
        })?;
    }
    Ok(extensibility)
}

pub fn get_field_id(field: &Field) -> Result<syn::Expr> {
    let mut result = Err(syn::Error::new(
        field.span(),
        r#"Field of mutable struct must define id attribute "#,
    ));

    if let Some(xtypes_attribute) = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("dust_dds"))
    {
        xtypes_attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("id") {
                result = Ok(meta.value()?.parse()?);
                Ok(())
            } else {
                Ok(())
            }
        })?;
    }

    result
}

pub fn field_has_key_attribute(field: &Field) -> syn::Result<bool> {
    let mut has_key = false;
    if let Some(xtypes_attribute) = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("dust_dds"))
    {
        xtypes_attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("key") {
                has_key = true;
                return Ok(());
            } else if meta.path.is_ident("id") {
                let id: Expr = meta.value()?.parse()?;
            }
            Ok(())
        })?;
    }
    Ok(has_key)
}
