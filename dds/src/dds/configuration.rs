use crate::infrastructure::error::{DdsError, DdsResult};

#[derive(Debug, PartialEq, Eq, Clone)]
/// This struct specifies the high-level configuration for the DustDDS library. The configuration can be set for use by the
/// [`dust_dds::domain::domain_participant_factory::DomainParticipantFactory::set_configuration`] method.
pub struct DustDdsConfiguration {
    domain_tag: String,
    interface_name: Option<String>,
    fragment_size: usize,
}

impl DustDdsConfiguration {
    /// Domain tag to use for the participants
    pub fn domain_tag(&self) -> &str {
        self.domain_tag.as_ref()
    }

    /// Network interface name to use for discovery
    pub fn interface_name(&self) -> Option<&String> {
        self.interface_name.as_ref()
    }

    /// Maximum size for the data fragments. Types with serialized data above this size will be transmitted as fragments.
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

/// Builder for the ['DustDdsConfiguration']
#[derive(Default)]
pub struct DustDdsConfigurationBuilder {
    configuration: DustDdsConfiguration,
}

impl DustDdsConfigurationBuilder {
    /// Construct a configuration builder with all the default options.
    pub fn new() -> Self {
        Self {
            configuration: Default::default(),
        }
    }

    /// Build a new configuration
    pub fn build(self) -> DustDdsConfiguration {
        self.configuration
    }

    /// Set the domain tag to use for the participants
    pub fn domain_tag(mut self, domain_tag: String) -> DdsResult<Self> {
        self.configuration.domain_tag = domain_tag;
        Ok(self)
    }

    /// Set the network interface name to use for discovery
    pub fn interface_name(mut self, interface_name: Option<String>) -> DdsResult<Self> {
        self.configuration.interface_name = interface_name;
        Ok(self)
    }

    /// Set the maximum size for the data fragments. Types with serialized data above this size will be transmitted as fragments.
    pub fn fragment_size(mut self, fragment_size: usize) -> DdsResult<Self> {
        let fragment_size_range = 8..=65000;
        if !fragment_size_range.contains(&fragment_size) {
            Err(DdsError::Error(format!(
                "Interface size out of range. Value must be between in {:?}",
                fragment_size_range
            )))
        } else {
            self.configuration.fragment_size = fragment_size;
            Ok(self)
        }
    }
}
