use syn::{DeriveInput, Expr, Field, Result, Variant, spanned::Spanned};

struct UnknownAttributeError;

impl std::fmt::Display for UnknownAttributeError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "unknown attribute".fmt(f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TryConstructKind {
    Discard,
    UseDefault,
    Trim,
}

pub struct StructureMemberAttributes {
    pub id: Option<Expr>,
    pub key: bool,
    pub optional: bool,
    pub non_serialized: bool,
    pub external: bool,
    pub hashid: bool,
    pub default_value: Option<Expr>,
    pub try_construct: Option<TryConstructKind>,
}

pub fn get_structure_member_attributes(field: &Field) -> Result<StructureMemberAttributes> {
    let mut id = None;
    let mut key = false;
    let mut optional = false;
    let mut default_value = None;
    let mut non_serialized = false;
    let mut external = false;
    let mut hashid = false;
    let mut try_construct = None;

    if let Some(xtypes_attribute) = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("dust_dds"))
    {
        xtypes_attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("id") {
                id = Some(meta.value()?.parse()?);
                Ok(())
            } else if meta.path.is_ident("key") {
                key = true;
                Ok(())
            } else if meta.path.is_ident("default_value") {
                default_value = Some(meta.value()?.parse()?);
                Ok(())
            } else if meta.path.is_ident("optional") {
                optional = true;
                Ok(())
            } else if meta.path.is_ident("non_serialized") {
                non_serialized = true;
                Ok(())
            } else if meta.path.is_ident("external") {
                external = true;
                Ok(())
            } else if meta.path.is_ident("hashid") {
                hashid = true;
                Ok(())
            } else if meta.path.is_ident("try_construct") {
               let format_str: syn::LitStr = meta.value()?.parse()?;
                match format_str.value().as_ref() {
                    "DISCARD" => {
                        try_construct = Some(TryConstructKind::Discard);
                        Ok(())
                    }
                    "USE_DEFAULT" => {
                        try_construct = Some(TryConstructKind::UseDefault);
                        Ok(())
                    }
                    "TRIM" => {
                        try_construct = Some(TryConstructKind::Trim);
                        Ok(())
                    }
                    _ => Err(meta.error(r#"Invalid try_construct specified. Valid options are "DISCARD", "USE_DEFAULT", "TRIM". "#))
                } 
            } else {
                Err(meta.error(UnknownAttributeError))
            }
        })?;
    }

    Ok(StructureMemberAttributes {
        id,
        key,
        optional,
        non_serialized,
        external,
        hashid,
        default_value,
        try_construct,
    })
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Extensibility {
    Final,
    Appendable,
    Mutable,
}

pub struct StructAttributes {
    pub name: String,
    pub extensibility: Extensibility,
    pub is_nested: bool,
    pub base_type: Option<syn::Type>,
}

pub fn get_struct_attributes(input: &DeriveInput) -> Result<StructAttributes> {
    let mut name = input.ident.to_string();
    let mut extensibility = Extensibility::Final;
    let mut is_nested = false;
    let mut base_type = None;

    if let Some(xtypes_attribute) = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("dust_dds"))
    {
        xtypes_attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                name = meta.value()?.parse::<syn::LitStr>()?.value();
                Ok(())
            } else if meta.path.is_ident("base_type") {
                base_type = Some(meta.value()?.parse::<syn::Type>()?);
                Ok(())
            } else if meta.path.is_ident("extensibility") {
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
                    _ => Err(meta.error(r#"Invalid format specified. Valid options are "final", "appendable", "mutable". "#))
                }
            } else if meta.path.is_ident("nested") {
                is_nested = true;
                Ok(())
            } else {
                Err(meta.error(UnknownAttributeError))
            }
        })?;
    }

    Ok(StructAttributes {
        name,
        extensibility,
        is_nested,
        base_type,
    })
}

pub enum BitBound {
    I8,
    I16,
    I32,
}

pub struct EnumeratedTypeAttributes {
    pub name: String,
    pub is_nested: bool,
    pub bit_bound: BitBound,
}

pub fn get_enumerated_type_attributes(input: &DeriveInput) -> Result<EnumeratedTypeAttributes> {
    let mut name = input.ident.to_string();
    let mut is_nested = false;
    let mut bit_bound = BitBound::I32;

    if let Some(xtypes_attribute) = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("dust_dds"))
    {
        xtypes_attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                name = meta.value()?.parse::<syn::LitStr>()?.value();
                Ok(())
            } else if meta.path.is_ident("nested") {
                is_nested = true;
                Ok(())
            } else if meta.path.is_ident("bit_bound") {
                let format_str: syn::LitStr = meta.value()?.parse()?;
                match format_str.value().as_ref() {
                    "8" => {
                        bit_bound = BitBound::I8;
                        Ok(())
                    }
                    "16" => {
                        bit_bound = BitBound::I16;
                        Ok(())
                    }
                    "32" => {
                        bit_bound = BitBound::I32;
                        Ok(())
                    }
                    _ => Err(meta.error(
                        r#"Invalid bit_bound specified. Valid options are "8", "16", "32". "#,
                    )),
                }
            } else {
                Err(meta.error(UnknownAttributeError))
            }
        })?;
    }

    Ok(EnumeratedTypeAttributes {
        name,
        is_nested,
        bit_bound,
    })
}

pub struct UnionAttributes {
    pub name: String,
    pub extensibility: Extensibility,
    pub is_nested: bool,
    pub discriminator_type: syn::Type,
    pub is_discriminator_key: bool,
}

pub fn get_union_type_attributes(input: &DeriveInput) -> Result<UnionAttributes> {
    let mut name = input.ident.to_string();
    let mut extensibility = Extensibility::Final;
    let mut is_nested = false;
    let mut is_discriminator_key = false;
    let mut discriminator_type = None;

    if let Some(xtypes_attribute) = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("dust_dds"))
    {
        xtypes_attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                name = meta.value()?.parse::<syn::LitStr>()?.value();
                Ok(())
            } else if meta.path.is_ident("extensibility") {
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
                    _ => Err(meta.error(r#"Invalid format specified. Valid options are "final", "appendable", "mutable". "#)),
                }
            } else if meta.path.is_ident("nested") {
                is_nested = true;
                Ok(())
            } else if meta.path.is_ident("switch") {
                let content;
                syn::parenthesized!(content in meta.input);
                let fork = content.fork();
                if let Ok(ident) = fork.parse::<syn::Ident>() {
                    if ident == "key" && fork.parse::<syn::Token![,]>().is_ok() {
                        is_discriminator_key = true;
                        let _: syn::Ident = content.parse()?;
                        let _: syn::Token![,] = content.parse()?;
                    }
                }
                discriminator_type = Some(content.parse::<syn::Type>()?);
                Ok(())
            } else {
                Err(meta.error(UnknownAttributeError))
            }
        })?;
    };

    let discriminator_type = discriminator_type.ok_or(syn::Error::new(
        input.span(),
        r#"Union must defined its discriminator type by adding #[dust_dds(switch(#type))] "#,
    ))?;

    Ok(UnionAttributes {
        name,
        extensibility,
        is_nested,
        discriminator_type,
        is_discriminator_key,
    })
}

pub struct UnionVariantAttributes {
    pub case: Vec<Expr>,
    pub is_default: bool,
}

pub fn get_union_variant_attributes(variant: &Variant) -> Result<UnionVariantAttributes> {
    let mut case = Vec::new();
    let mut is_default = false;

    if let Some(xtypes_attribute) = variant
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("dust_dds"))
    {
        xtypes_attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("case") {
                case.push(meta.value()?.parse()?);
                Ok(())
            } else if meta.path.is_ident("default") {
                is_default = true;
                Ok(())
            } else {
                Err(meta.error(UnknownAttributeError))
            }
        })?;
    }

    Ok(UnionVariantAttributes { case, is_default })
}
