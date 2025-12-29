use crate::infrastructure::error::DdsResult;
use alloc::string::{String, ToString};
use core::time::Duration;

#[derive(Debug, PartialEq, Eq, Clone)]
/// This struct specifies the high-level configuration for the DustDDS library. The configuration can be set for use by the
/// [`DomainParticipantFactory::set_configuration`](dust_dds::domain::domain_participant_factory::DomainParticipantFactory::set_configuration) method.
pub struct DustDdsConfiguration {
    domain_tag: String,
    participant_announcement_interval: Duration,
    participant_lease_duration: Duration,
}

impl DustDdsConfiguration {
    /// Domain tag to use for the participants
    pub fn domain_tag(&self) -> &str {
        self.domain_tag.as_ref()
    }

    /// Maximum interval at which the participant is announced on the network.
    pub fn participant_announcement_interval(&self) -> Duration {
        self.participant_announcement_interval
    }

    /// Duration after which a participant is considered dead if no announcements are received.
    pub fn participant_lease_duration(&self) -> Duration {
        self.participant_lease_duration
    }
}

impl Default for DustDdsConfiguration {
    fn default() -> Self {
        Self {
            domain_tag: "".to_string(),
            participant_announcement_interval: Duration::from_secs(5),
            participant_lease_duration: Duration::from_secs(100),
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
        Ok(self.configuration)
    }

    /// Set the domain tag to use for the participants
    pub fn domain_tag(mut self, domain_tag: String) -> Self {
        self.configuration.domain_tag = domain_tag;
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

    /// Set the duration after which a participant is considered dead if no announcements are received.
    /// This is the lease duration announced via SPDP.
    pub fn participant_lease_duration(mut self, participant_lease_duration: Duration) -> Self {
        self.configuration.participant_lease_duration = participant_lease_duration;
        self
    }
}
