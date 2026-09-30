use crate::{infrastructure::error::DdsResult, security::plugins::authentication::Authentication};
use alloc::{
    string::{String, ToString},
    sync::Arc,
};
use core::time::Duration;

#[derive(Clone)]
/// This struct specifies the high-level configuration for the DustDDS library. The configuration can be set for use by the
/// [`DomainParticipantFactory::set_configuration`](dust_dds::domain::domain_participant_factory::DomainParticipantFactory::set_configuration) method.
pub struct DustDdsConfiguration {
    domain_tag: String,
    participant_announcement_interval: Duration,
    enable_type_information: bool,
    authentication_plugin: Option<Arc<dyn Authentication>>,
}

impl core::fmt::Debug for DustDdsConfiguration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DustDdsConfiguration")
            .field("domain_tag", &self.domain_tag)
            .field(
                "participant_announcement_interval",
                &self.participant_announcement_interval,
            )
            .field("enable_type_information", &self.enable_type_information)
            .field(
                "authentication_plugin",
                &self.authentication_plugin.as_ref().map(|_| "..."),
            )
            .finish()
    }
}

impl PartialEq for DustDdsConfiguration {
    fn eq(&self, other: &Self) -> bool {
        self.domain_tag == other.domain_tag
            && self.participant_announcement_interval == other.participant_announcement_interval
            && self.enable_type_information == other.enable_type_information
            && match (&self.authentication_plugin, &other.authentication_plugin) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}

impl Eq for DustDdsConfiguration {}

impl DustDdsConfiguration {
    /// Domain tag to use for the participants
    pub fn domain_tag(&self) -> &str {
        self.domain_tag.as_ref()
    }

    /// Maximum interval at which the participant is announced on the network.
    pub fn participant_announcement_interval(&self) -> Duration {
        self.participant_announcement_interval
    }

    /// Enable type information which allows exchanging details about the types of different topics
    pub fn enable_type_information(&self) -> bool {
        self.enable_type_information
    }

    /// Authentication plugin to use for participant security
    pub fn authentication_plugin(&self) -> Option<Arc<dyn Authentication>> {
        self.authentication_plugin.clone()
    }
}

impl Default for DustDdsConfiguration {
    fn default() -> Self {
        Self {
            domain_tag: "".to_string(),
            participant_announcement_interval: Duration::from_secs(5),
            enable_type_information: true,
            authentication_plugin: None,
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

    /// Set whether type information should be enabled or disabled
    pub fn enable_type_information(mut self, enable_type_information: bool) -> Self {
        self.configuration.enable_type_information = enable_type_information;
        self
    }

    /// Set the authentication plugin to use for participant security
    pub fn authentication_plugin(mut self, authentication_plugin: Arc<dyn Authentication>) -> Self {
        self.configuration.authentication_plugin = Some(authentication_plugin);
        self
    }
}
