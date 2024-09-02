use syn::{spanned::Spanned, DeriveInput, Result};

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
        .find(|attr| attr.path().is_ident("xtypes"))
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
