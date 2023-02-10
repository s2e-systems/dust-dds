use jsonschema::JSONSchema;
use schemars::{schema_for, JsonSchema};
use serde::Deserialize;
use std::str::FromStr;

use crate::infrastructure::error::{DdsError, DdsResult};

#[derive(Deserialize, JsonSchema, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct DustDdsConfiguration {
    /// # Domain tag
    /// Domain tag to use for the participant
    pub domain_tag: String,
    /// # Interface name
    /// Network interface name to use for discovery
    pub interface_name: Option<String>,
    /// # Fragment size
    /// Data is fragmented into max size of this
    #[schemars(range(min = 8))]
    pub fragment_size: usize,
}

impl Default for DustDdsConfiguration {
    fn default() -> Self {
        Self {
            domain_tag: "".to_string(),
            interface_name: None,
            fragment_size: 1344,
        }
    }
}

impl DustDdsConfiguration {
    pub fn try_from_str(configuration_json: &str) -> DdsResult<Self> {
        let root_schema = schema_for!(DustDdsConfiguration);
        let json_schema_str =
            serde_json::to_string(&root_schema).expect("Json schema could not be created");

        let schema = serde_json::value::Value::from_str(json_schema_str.as_str())
            .expect("Json schema not valid");
        let compiled_schema =
            JSONSchema::compile(&schema).expect("Json schema could not be compiled");

        let instance = serde_json::value::Value::from_str(configuration_json)
            .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))?;
        compiled_schema.validate(&instance).map_err(|errors| {
            DdsError::PreconditionNotMet(errors.map(|e| e.to_string()).collect())
        })?;
        serde_json::from_value(instance).map_err(|e| DdsError::PreconditionNotMet(e.to_string()))
    }
}

pub fn generate_dust_dds_configuration_schema() -> Result<String, std::io::Error> {
    let root_schema = schema_for!(DustDdsConfiguration);
    Ok(serde_json::to_string_pretty(&root_schema)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_configuration_json() {
        let configuration = DustDdsConfiguration::try_from_str(
            r#"{"domain_tag" : "from_configuration_json", "interface_name": "Wi-Fi"}"#,
        )
        .unwrap();
        assert_eq!(
            configuration,
            DustDdsConfiguration {
                domain_tag: "from_configuration_json".to_string(),
                interface_name: Some("Wi-Fi".to_string()),
                fragment_size: 1344
            }
        );
    }
}
