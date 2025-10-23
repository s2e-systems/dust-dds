use syn::{Expr, Field};

pub struct FieldAttributes {
    pub key: bool,
    pub optional: bool,
    pub id: Option<Expr>,
    pub default_value: Option<Expr>,
    pub non_serialized: bool,
}

pub fn get_field_attributes(field: &Field) -> syn::Result<FieldAttributes> {
    let mut key = false;
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
                key = true;
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
