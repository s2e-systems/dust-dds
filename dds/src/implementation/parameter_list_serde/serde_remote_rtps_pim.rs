use crate::implementation::data_representation_builtin_endpoints::parameter_id_values::{
    DEFAULT_DOMAIN_TAG, DEFAULT_EXPECTS_INLINE_QOS,
};

#[derive(Debug, PartialEq, serde::Serialize, derive_more::From)]
pub struct ExpectsInlineQosSerialize<'a>(pub &'a bool);
impl Default for ExpectsInlineQosSerialize<'_> {
    fn default() -> Self {
        Self(&DEFAULT_EXPECTS_INLINE_QOS)
    }
}
#[derive(Debug, PartialEq, serde::Deserialize, derive_more::Into)]
pub struct ExpectsInlineQosDeserialize(pub bool);
impl Default for ExpectsInlineQosDeserialize {
    fn default() -> Self {
        Self(DEFAULT_EXPECTS_INLINE_QOS)
    }
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct DomainTag<'a>(pub &'a str);
impl<'a> Default for DomainTag<'a> {
    fn default() -> Self {
        Self(DEFAULT_DOMAIN_TAG)
    }
}

#[derive(Debug, PartialEq, serde::Serialize, derive_more::From)]
pub struct DomainTagSerialize<'a>(pub &'a DomainTag<'a>);

#[derive(Debug, PartialEq, serde::Deserialize, derive_more::Into)]
pub struct DomainTagDeserialize(pub String);
impl Default for DomainTagDeserialize {
    fn default() -> Self {
        Self(DEFAULT_DOMAIN_TAG.to_string())
    }
}
