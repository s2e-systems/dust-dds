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
                    "final" => {
                        extensibility = Extensibility::Final;
                        Ok(())
                    }
                    "appendable" => {
                        extensibility = Extensibility::Appendable;
                        Ok(())
                    }
                    "mutable" => {
                        extensibility = Extensibility::Mutable;
                        Ok(())
                    }
                    _ => Err(syn::Error::new(
                        meta.path.span(),
                        r#"Invalid format specified. Valid options are "final", "appendable", "mutable". "#,
                    )),
                }
            } else {
                Ok(())
            }
        })?;
    }
    Ok(extensibility)
}

pub struct FieldAttributes {
    pub key: bool,
    pub id: Option<Expr>,
}

pub fn get_field_attributes(field: &Field) -> syn::Result<FieldAttributes> {
    let mut key = false;
    let mut id = None;
    if let Some(xtypes_attribute) = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("dust_dds"))
    {
        xtypes_attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("key") {
                key = true;
            } else if meta.path.is_ident("id") {
                id = Some(meta.value()?.parse()?);
            }
            Ok(())
        })?;
    }
    Ok(FieldAttributes { key, id })
}
