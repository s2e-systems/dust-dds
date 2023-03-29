use schemars::JsonSchema;
use serde::Deserialize;

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
