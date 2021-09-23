use rust_dds_api::infrastructure::qos_policy::UserDataQosPolicy;


#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "UserDataQosPolicy")]
pub struct UserDataQosPolicyDef<'a> {
    pub value: &'a [u8],
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct UserDataQosPolicySerdeSerialize<'a>(#[serde(with = "UserDataQosPolicyDef")] pub &'a UserDataQosPolicy<'a>);
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct UserDataQosPolicySerdeDeserialize<'a>(#[serde(with = "UserDataQosPolicyDef")] #[serde(borrow)] UserDataQosPolicy<'a>);