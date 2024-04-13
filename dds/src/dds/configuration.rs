use std::time::Duration;

use crate::infrastructure::error::{DdsError, DdsResult};

#[derive(Debug, PartialEq, Eq, Clone)]
/// This struct specifies the high-level configuration for the DustDDS library. The configuration can be set for use by the
/// [`DomainParticipantFactory::set_configuration`](dust_dds::domain::domain_participant_factory::DomainParticipantFactory::set_configuration) method.
pub struct DustDdsConfiguration {
    domain_tag: String,
    interface_name: Option<String>,
    fragment_size: usize,
    udp_receive_buffer_size: Option<usize>,
    participant_announcement_interval: Duration,
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

    /// Receive buffer size used for UDP socket. [`None`] means the OS default value
    pub fn udp_receive_buffer_size(&self) -> Option<usize> {
        self.udp_receive_buffer_size
    }

    /// Maximum interval at which the participant is announced on the network.
    pub fn participant_announcement_interval(&self) -> Duration {
        self.participant_announcement_interval
    }
}

impl Default for DustDdsConfiguration {
    fn default() -> Self {
        Self {
            domain_tag: "".to_string(),
            interface_name: None,
            fragment_size: 1344,
            udp_receive_buffer_size: None,
            participant_announcement_interval: Duration::from_secs(5),
        }
    }
}

/// Builder for the [`DustDdsConfiguration`]
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
    pub fn build(self) -> DdsResult<DustDdsConfiguration> {
        let fragment_size_range = 8..=65000;
        if !fragment_size_range.contains(&self.configuration.fragment_size) {
            Err(DdsError::Error(format!(
                "Interface size out of range. Value must be between in {:?}",
                fragment_size_range
            )))
        } else {
            Ok(self.configuration)
        }
    }

    /// Set the domain tag to use for the participants
    pub fn domain_tag(mut self, domain_tag: String) -> Self {
        self.configuration.domain_tag = domain_tag;
        self
    }

    /// Set the network interface name to use for discovery
    pub fn interface_name(mut self, interface_name: Option<String>) -> Self {
        self.configuration.interface_name = interface_name;
        self
    }

    /// Set the maximum size for the data fragments. Types with serialized data above this size will be transmitted as fragments.
    pub fn fragment_size(mut self, fragment_size: usize) -> Self {
        self.configuration.fragment_size = fragment_size;
        self
    }

    /// Set the value of the SO_RCVBUF option on the UDP socket. [`None`] corresponds to the OS default
    pub fn udp_receive_buffer_size(mut self, udp_receive_buffer_size: Option<usize>) -> Self {
        self.configuration.udp_receive_buffer_size = udp_receive_buffer_size;
        self
    }

    /// Set the maximum interval at which the participant is announced on the network. This corresponds to the time
    /// between SPDP messages.
    pub fn participant_announcement_interval(
        mut self,
        participant_announcement_interval: Duration,
    ) -> Self {
        self.configuration.participant_announcement_interval = participant_announcement_interval;
        self
    }
}
