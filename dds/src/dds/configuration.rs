use crate::infrastructure::error::{DdsError, DdsResult};

#[derive(Debug, PartialEq, Eq, Clone)]
/// # Dust DDS Configuration
/// The environment DUST_DDS_CONFIGURATION variable can be set
/// as a json to modify the default configuration.
/// E.g. $Env:DUST_DDS_CONFIGURATION='{"interface_name":"Wi-Fi"}'
pub struct DustDdsConfiguration {
    /// # Domain tag
    /// Domain tag to use for the participant
    domain_tag: String,
    /// # Interface name
    /// Network interface name to use for discovery
    interface_name: Option<String>,
    /// # Fragment size
    /// Data is fragmented into max size of this
    fragment_size: usize,
}

impl DustDdsConfiguration {
    pub fn domain_tag(&self) -> &str {
        self.domain_tag.as_ref()
    }

    pub fn interface_name(&self) -> Option<&String> {
        self.interface_name.as_ref()
    }

    pub fn fragment_size(&self) -> usize {
        self.fragment_size
    }
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

pub struct DustDdsConfigurationBuilder {
    configuration: DustDdsConfiguration,
}

impl DustDdsConfigurationBuilder {
    pub fn new() -> Self {
        Self {
            configuration: Default::default(),
        }
    }

    pub fn build(self) -> DustDdsConfiguration {
        self.configuration
    }

    pub fn domain_tag(mut self, domain_tag: String) -> DdsResult<Self> {
        self.configuration.domain_tag = domain_tag;
        Ok(self)
    }

    pub fn interface_name(mut self, interface_name: Option<String>) -> DdsResult<Self> {
        self.configuration.interface_name = interface_name;
        Ok(self)
    }

    pub fn fragment_size(mut self, fragment_size: usize) -> DdsResult<Self> {
        if fragment_size < 8 || fragment_size > 65000 {
            Err(DdsError::Error(
                "Interface size out of range. Value must be between 8 and 65000".to_string(),
            ))
        } else {
            self.configuration.fragment_size = fragment_size;
            Ok(self)
        }
    }
}
