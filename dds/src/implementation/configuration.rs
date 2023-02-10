use jsonschema::JSONSchema;
use schemars::{schema_for, JsonSchema};
use serde::Deserialize;
use std::str::FromStr;


use crate::infrastructure::error::{DdsError, DdsResult};

fn default_domain_tag() -> String {
    "".to_string()
}

fn default_interface_name() -> Option<String> {
    None
}

fn default_fragment_size() -> usize {
    1344
}

#[derive(Deserialize, JsonSchema, Debug, PartialEq, Eq)]
pub struct DustDdsConfiguration {
    /// # Domain tag
    /// Domain tag to use for the participant
    #[serde(default = "default_domain_tag")]
    pub domain_tag: String,
    /// # Interface name
    /// Network interface name to use for discovery
    #[serde(default = "default_interface_name")]
    pub interface_name: Option<String>,
    /// # Fragment size
    /// Data is fragmented into max size of this
    #[serde(default = "default_fragment_size")]
    #[schemars(range(min = 8))]
    pub fragment_size: usize,
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
    let json_schema_str_pretty = serde_json::to_string_pretty(&root_schema).unwrap();
    Ok(json_schema_str_pretty)
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
