use std::str::FromStr;
use syn::{DeriveInput, Expr, Field, Result, Variant, spanned::Spanned};

const DUST_DDS_ATTR: &str = "dust_dds";

trait OptionExt {
    fn err_if_some<E, F>(self, f: F) -> std::result::Result<(), E>
    where
        F: FnOnce() -> E;
}

impl<T> OptionExt for Option<T> {
    #[inline]
    fn err_if_some<E, F>(self, f: F) -> std::result::Result<(), E>
    where
        F: FnOnce() -> E,
    {
        match self {
            Some(_) => Err(f()),
            None => Ok(()),
        }
    }
}

struct UnknownAttributeError;

impl std::fmt::Display for UnknownAttributeError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "unknown attribute".fmt(f)
    }
}

struct DuplicateAttributeError;

impl std::fmt::Display for DuplicateAttributeError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "duplicate attribute".fmt(f)
    }
}

#[derive(Default)]
pub enum TryConstructKind {
    #[default]
    Discard,
    UseDefault,
    Trim,
}

pub struct TryConstructKindParseError;

impl std::fmt::Display for TryConstructKindParseError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        r#"Invalid try_construct specified. Valid options are "DISCARD", "USE_DEFAULT", "TRIM". "#
            .fmt(f)
    }
}

impl FromStr for TryConstructKind {
    type Err = TryConstructKindParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "DISCARD" => Ok(Self::Discard),
            "USE_DEFAULT" => Ok(Self::UseDefault),
            "TRIM" => Ok(Self::Trim),
            _ => Err(TryConstructKindParseError),
        }
    }
}

pub struct StructureMemberAttributes {
    pub id: Option<Expr>,
    pub key: bool,
    pub optional: bool,
    pub non_serialized: bool,
    pub external: bool,
    pub hashid: Option<String>,
    pub default_value: Option<Expr>,
    pub try_construct: TryConstructKind,
}

pub fn get_structure_member_attributes(field: &Field) -> Result<StructureMemberAttributes> {
    let mut id = None;
    let mut key = None;
    let mut optional = None;
    let mut default_value = None;
    let mut non_serialized = None;
    let mut external = None;
    let mut hashid = None;
    let mut try_construct = None;

    for attr in field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident(DUST_DDS_ATTR))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("id") {
                id.replace(meta.value()?.parse()?)
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else if meta.path.is_ident("key") {
                key.replace(true)
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else if meta.path.is_ident("default_value") {
                default_value
                    .replace(meta.value()?.parse()?)
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else if meta.path.is_ident("optional") {
                optional
                    .replace(true)
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else if meta.path.is_ident("non_serialized") {
                non_serialized
                    .replace(true)
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else if meta.path.is_ident("external") {
                external
                    .replace(true)
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else if meta.path.is_ident("hashid") {
                let value = match meta.value() {
                    Ok(value) => value.parse::<syn::LitStr>()?.value(),
                    Err(_) => String::default(),
                };

                hashid
                    .replace(value)
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else if meta.path.is_ident("try_construct") {
                let value = meta
                    .value()?
                    .parse::<syn::LitStr>()?
                    .value()
                    .parse()
                    .map_err(|err| meta.error(err))?;

                try_construct
                    .replace(value)
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else {
                Err(meta.error(UnknownAttributeError))
            }
        })?;
    }

    Ok(StructureMemberAttributes {
        id,
        key: key.unwrap_or_default(),
        optional: optional.unwrap_or_default(),
        non_serialized: non_serialized.unwrap_or_default(),
        external: external.unwrap_or_default(),
        hashid,
        default_value,
        try_construct: try_construct.unwrap_or_default(),
    })
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum Extensibility {
    #[default]
    Final,
    Appendable,
    Mutable,
}

pub struct ExtensibilityParseError;

impl std::fmt::Display for ExtensibilityParseError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        r#"Invalid extensibility specified. Valid options are "final", "appendable", "mutable". "#
            .fmt(f)
    }
}

impl FromStr for Extensibility {
    type Err = ExtensibilityParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "final" => Ok(Self::Final),
            "appendable" => Ok(Self::Appendable),
            "mutable" => Ok(Self::Mutable),
            _ => Err(ExtensibilityParseError),
        }
    }
}

pub struct StructAttributes {
    pub name: String,
    pub extensibility: Extensibility,
    pub is_nested: bool,
    pub is_autoid_hash: bool,
    pub base_type: Option<syn::Type>,
}

pub fn get_struct_attributes(input: &DeriveInput) -> Result<StructAttributes> {
    let mut name = None;
    let mut extensibility = None;
    let mut is_nested = None;
    let mut is_autoid_hash = None;
    let mut base_type = None;

    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident(DUST_DDS_ATTR))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                name.replace(meta.value()?.parse::<syn::LitStr>()?.value())
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else if meta.path.is_ident("base_type") {
                base_type
                    .replace(meta.value()?.parse()?)
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else if meta.path.is_ident("extensibility") {
                let value = meta
                    .value()?
                    .parse::<syn::LitStr>()?
                    .value()
                    .parse()
                    .map_err(|err| meta.error(err))?;

                extensibility
                    .replace(value)
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else if meta.path.is_ident("nested") {
                is_nested
                    .replace(true)
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else if meta.path.is_ident("autoid") {
                match meta.value()?.parse::<syn::LitStr>()?.value().as_str() {
                    "hash" => is_autoid_hash
                        .replace(true)
                        .err_if_some(|| meta.error(DuplicateAttributeError)),
                    _ => Err(meta
                        .error(r#"Invalid autoid attribute specified. Valid option is "hash". "#)),
                }
            } else {
                Err(meta.error(UnknownAttributeError))
            }
        })?;
    }

    Ok(StructAttributes {
        name: name.unwrap_or_else(|| input.ident.to_string()),
        extensibility: extensibility.unwrap_or_default(),
        is_nested: is_nested.unwrap_or_default(),
        is_autoid_hash: is_autoid_hash.unwrap_or_default(),
        base_type,
    })
}

#[derive(Default)]
pub enum BitBound {
    I8,
    I16,
    #[default]
    I32,
}

pub struct BitBoundParseError;

impl std::fmt::Display for BitBoundParseError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        r#"Invalid bit_bound specified. Valid options are "8", "16", "32". "#.fmt(f)
    }
}

impl FromStr for BitBound {
    type Err = BitBoundParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "8" => Ok(Self::I8),
            "16" => Ok(Self::I16),
            "32" => Ok(Self::I32),
            _ => Err(BitBoundParseError),
        }
    }
}

pub struct EnumeratedTypeAttributes {
    pub name: String,
    pub is_nested: bool,
    pub bit_bound: BitBound,
}

pub fn get_enumerated_type_attributes(input: &DeriveInput) -> Result<EnumeratedTypeAttributes> {
    let mut name = None;
    let mut is_nested = None;
    let mut bit_bound = None;

    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident(DUST_DDS_ATTR))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                name.replace(meta.value()?.parse::<syn::LitStr>()?.value())
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else if meta.path.is_ident("nested") {
                is_nested
                    .replace(true)
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else if meta.path.is_ident("bit_bound") {
                let value = meta
                    .value()?
                    .parse::<syn::LitStr>()?
                    .value()
                    .parse()
                    .map_err(|err| meta.error(err))?;

                bit_bound
                    .replace(value)
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else {
                Err(meta.error(UnknownAttributeError))
            }
        })?;
    }

    Ok(EnumeratedTypeAttributes {
        name: name.unwrap_or_else(|| input.ident.to_string()),
        is_nested: is_nested.unwrap_or_default(),
        bit_bound: bit_bound.unwrap_or_default(),
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
    let mut name = None;
    let mut extensibility = None;
    let mut is_nested = None;
    let mut is_discriminator_key = None;
    let mut discriminator_type = None;

    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident(DUST_DDS_ATTR))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                name.replace(meta.value()?.parse::<syn::LitStr>()?.value())
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else if meta.path.is_ident("extensibility") {
                let value = meta
                    .value()?
                    .parse::<syn::LitStr>()?
                    .value()
                    .parse()
                    .map_err(|err| meta.error(err))?;

                extensibility
                    .replace(value)
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else if meta.path.is_ident("nested") {
                is_nested
                    .replace(true)
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else if meta.path.is_ident("switch") {
                let content;
                syn::parenthesized!(content in meta.input);
                let fork = content.fork();
                if let Ok(ident) = fork.parse::<syn::Ident>() {
                    if ident == "key" && fork.parse::<syn::Token![,]>().is_ok() {
                        is_discriminator_key
                            .replace(true)
                            .err_if_some(|| meta.error(DuplicateAttributeError))?;
                        let _: syn::Ident = content.parse()?;
                        let _: syn::Token![,] = content.parse()?;
                    }
                }

                discriminator_type
                    .replace(content.parse()?)
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else {
                Err(meta.error(UnknownAttributeError))
            }
        })?;
    }

    let discriminator_type = discriminator_type.ok_or(syn::Error::new(
        input.span(),
        r#"Union must defined its discriminator type by adding #[dust_dds(switch(#type))] "#,
    ))?;

    Ok(UnionAttributes {
        name: name.unwrap_or_else(|| input.ident.to_string()),
        extensibility: extensibility.unwrap_or_default(),
        is_nested: is_nested.unwrap_or_default(),
        discriminator_type,
        is_discriminator_key: is_discriminator_key.unwrap_or_default(),
    })
}

pub struct UnionVariantAttributes {
    pub case: Vec<Expr>,
    pub is_default: bool,
}

pub fn get_union_variant_attributes(variant: &Variant) -> Result<UnionVariantAttributes> {
    let mut case = Vec::new();
    let mut is_default = None;

    for attr in variant
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident(DUST_DDS_ATTR))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("case") {
                case.push(meta.value()?.parse()?);
                Ok(())
            } else if meta.path.is_ident("default") {
                is_default
                    .replace(true)
                    .err_if_some(|| meta.error(DuplicateAttributeError))
            } else {
                Err(meta.error(UnknownAttributeError))
            }
        })?;
    }

    Ok(UnionVariantAttributes {
        case,
        is_default: is_default.unwrap_or_default(),
    })
}
