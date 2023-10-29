use syn::{spanned::Spanned, DataStruct, Field};

pub fn field_has_key_attribute(field: &Field) -> bool {
    field.attrs.iter().any(|attr| attr.path().is_ident("key"))
}

pub fn struct_has_key(data_struct: &DataStruct) -> bool {
    data_struct.fields.iter().any(field_has_key_attribute)
}

pub fn get_parameter_id_attribute(field: &Field) -> syn::Result<syn::Expr> {
    let parameter_attribute = field
        .attrs
        .iter()
        .find(|a| a.path().is_ident("parameter"))
        .ok_or(syn::Error::new(
            field.span(),
            "Field missing #[parameter] attribute",
        ))?;
    let mut value: Option<syn::Expr> = None;
    parameter_attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("id") {
            value = Some(meta.value()?.parse()?);
        } else {
            // Ignore the tokens after the =
            let _: syn::Expr = meta.value()?.parse()?;
        }
        Ok(())
    })?;

    value.ok_or(syn::Error::new(
        parameter_attribute.span(),
        "\"id\" attribute not found",
    ))
}

pub fn get_parameter_default_attribute(field: &Field) -> syn::Result<Option<syn::Expr>> {
    let parameter_attribute = field
        .attrs
        .iter()
        .find(|a| a.path().is_ident("parameter"))
        .ok_or(syn::Error::new(
            field.span(),
            "Field missing #[parameter] attribute",
        ))?;
    let mut value: Option<syn::Expr> = None;
    parameter_attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("default") {
            value = Some(meta.value()?.parse()?);
        } else {
            // Ignore the tokens after the =
            let _: syn::Expr = meta.value()?.parse()?;
        }
        Ok(())
    })?;

    Ok(value)
}
