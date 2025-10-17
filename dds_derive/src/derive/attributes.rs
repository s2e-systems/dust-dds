use syn::{spanned::Spanned, DeriveInput, Expr, Field, Result};

pub enum Extensibility {
    Final,
    Appendable,
    Mutable,
}

fn parse_extensibility(value: &str, span: proc_macro2::Span) -> Result<Extensibility> {
    match value.to_ascii_lowercase().as_str() {
        "final" => Ok(Extensibility::Final),
        "appendable" => Ok(Extensibility::Appendable),
        "mutable" => Ok(Extensibility::Mutable),
        other => Err(syn::Error::new(
            span,
            format!(
                "Invalid extensibility: `{}`. Expected \"final\", \"appendable\", or \"mutable\".",
                other
            ),
        )),
    }
}

pub fn get_input_extensibility(input: &DeriveInput) -> Result<Extensibility> {
    let mut extensibility = Extensibility::Final;

    for attr in input.attrs.iter().filter(|a| a.path().is_ident("dust_dds")) {
        attr.parse_nested_meta(|meta| {
            if let Some(ident) = meta.path.get_ident() {
                if ident == "extensibility" {
                    let format_str: syn::LitStr = meta.value()?.parse()?;
                    extensibility = parse_extensibility(&format_str.value(), format_str.span())?;
                } else if ident.eq_ignore_ascii_case("final") {
                    extensibility = Extensibility::Final;
                } else if ident.eq_ignore_ascii_case("appendable") {
                    extensibility = Extensibility::Appendable;
                } else if ident.eq_ignore_ascii_case("mutable") {
                    extensibility = Extensibility::Mutable;
                }
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
    pub default_value: Option<Expr>,
    pub non_serialized: bool,
    pub hashid: bool,
    pub must_understand: bool,
}

pub fn get_field_attributes(field: &Field) -> Result<FieldAttributes> {
    let mut attrs = FieldAttributes::default();

    for attr in field.attrs.iter().filter(|a| a.path().is_ident("dust_dds")) {
        attr.parse_nested_meta(|meta| {
            if let Some(ident) = meta.path.get_ident() {
                if ident == "key" {
                    attrs.key = true;
                } else if ident.eq_ignore_ascii_case("optional") {
                    attrs.optional = true;
                } else if ident == "id" {
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
                } else if ident == "default_value" {
                    attrs.default_value = Some(meta.value()?.parse()?);
                } else if ident == "non_serialized" {
                    attrs.non_serialized = true;
                } else if ident == "hashid" {
                    attrs.hashid = true;
                } else if ident == "must_understand" {
                    attrs.must_understand = true;
                } else {
                    return Err(syn::Error::new(
                        meta.path.span(),
                        format!("Unknown dust_dds field attribute: `{}`", ident),
                    ));
                }
            }

            Ok(())
        })?;
    }

    Ok(attrs)
}
```