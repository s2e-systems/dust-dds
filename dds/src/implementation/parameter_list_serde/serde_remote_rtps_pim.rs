use crate::implementation::data_representation_builtin_endpoints::parameter_id_values::DEFAULT_EXPECTS_INLINE_QOS;

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub struct ExpectsInlineQosSerialize<'a>(pub &'a bool);
impl Default for ExpectsInlineQosSerialize<'_> {
    fn default() -> Self {
        Self(&DEFAULT_EXPECTS_INLINE_QOS)
    }
}
#[derive(Debug, PartialEq, Eq, serde::Deserialize, derive_more::Into)]
pub struct ExpectsInlineQosDeserialize(pub bool);
impl Default for ExpectsInlineQosDeserialize {
    fn default() -> Self {
        Self(DEFAULT_EXPECTS_INLINE_QOS)
    }
}
