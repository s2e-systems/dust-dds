use syn::{spanned::Spanned, DeriveInput, Expr, Field, Result};

pub enum Extensibility {
    Final,
    Appendable,
    Mutable,
}

pub fn get_input_extensibility(input: &DeriveInput) -> Result<Extensibility> {
    let mut extensibility = Extensibility::Final;

    for attr in input.attrs.iter().filter(|a| a.path().is_ident("dust_dds")) {
        attr.parse_nested_meta(|meta| {
            let name = meta.path.get_ident().map(|id| id.to_string());

            match name.as_deref() {
                Some("extensibility") => {
                    let format_str: syn::LitStr = meta.value()?.parse()?;
                    extensibility = match format_str.value().as_str() {
                        "Final" => Extensibility::Final,
                        "Appendable" => Extensibility::Appendable,
                        "Mutable" => Extensibility::Mutable,
                        other => {
                            return Err(syn::Error::new(
                                format_str.span(),
                                format!("Invalid extensibility: `{}`. Use \"Final\", \"Appendable\", or \"Mutable\"", other),
                            ));
                        }
                    };
                }
                Some("final") => extensibility = Extensibility::Final,
                Some("appendable") => extensibility = Extensibility::Appendable,
                Some("mutable") => extensibility = Extensibility::Mutable,
                _ => {}
            }

            Ok(())
        })?;
    }

    Ok(extensibility)
}


#[derive(Default)]
pub struct FieldAttributes {
    pub key: bool,
    pub id: Option<Expr>,
    pub optional: bool,
    pub hashid: bool,
    pub must_understand: bool,
}


pub fn get_field_attributes(field: &Field) -> Result<FieldAttributes> {
    let mut attrs = FieldAttributes::default();

    for attr in field.attrs.iter().filter(|a| a.path().is_ident("dust_dds")) {
        attr.parse_nested_meta(|meta| {
            let name = meta.path.get_ident().map(|id| id.to_string());

            match name.as_deref() {
                Some("key") => {
                    attrs.key = true;
                }
                Some("optional") | Some("Optional") => {
                    attrs.optional = true;
                }
                Some("hashid") => {
                    attrs.hashid = true;
                }
                Some("must_understand") => {
                    attrs.must_understand = true;
                }
                Some("id") => {
                    let expr: Expr = meta.value()?.parse()?;
                    if let Expr::Lit(expr_lit) = &expr {
                        if let syn::Lit::Int(_) = &expr_lit.lit {
                            attrs.id = Some(expr);
                        } else {
                            return Err(syn::Error::new(expr.span(), "Expected integer literal for `id`"));
                        }
                    } else {
                        return Err(syn::Error::new(expr.span(), "Expected literal expression for `id`"));
                    }
                }
                Some(other) => {
                    return Err(syn::Error::new(
                        meta.path.span(),
                        format!("Unknown dust_dds field attribute: `{}`", other),
                    ));
                }
                None => {}
            }

            Ok(())
        })?;
    }

    Ok(attrs)
}

