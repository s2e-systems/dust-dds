use jsonschema::JSONSchema;
use schemars::{schema_for, JsonSchema};
use serde::Deserialize;
use std::str::FromStr;

use std::io::prelude::*;

use crate::infrastructure::error::{DdsError, DdsResult};

fn default_domain_tag() -> String {
    "".to_string()
}

#[derive(Deserialize, JsonSchema, Debug, PartialEq, Eq)]
pub struct DustDdsConfiguration {
    #[serde(default = "default_domain_tag")]
    pub domain_tag: String,
}

impl Default for DustDdsConfiguration {
    fn default() -> Self {
        Self {
            domain_tag: default_domain_tag(),
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

    pub fn _write_schema_file() -> DdsResult<()> {
        let root_schema = schema_for!(DustDdsConfiguration);
        let json_schema_str_pretty = serde_json::to_string_pretty(&root_schema).unwrap();

        let mut file = std::fs::File::create("schema.json").unwrap();
        file.write_all(json_schema_str_pretty.as_bytes()).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_empty_configuration_json() {
        let configuration = DustDdsConfiguration::try_from_str("{}").unwrap();
        assert_eq!(configuration, DustDdsConfiguration::default())
    }

    #[test]
    fn from_configuration_json() {
        let configuration =
            DustDdsConfiguration::try_from_str(r#"{"domain_tag" : "from_configuration_json" }"#)
                .unwrap();
        assert_eq!(
            configuration,
            DustDdsConfiguration {
                domain_tag: "from_configuration_json".to_string()
            }
        );
    }
}
