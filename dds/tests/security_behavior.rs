mod utils;
use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        configuration::DustDdsConfiguration,
        domain::DomainId,
        listener::NO_LISTENER,
        qos::{DomainParticipantQos, QosKind},
        status::NO_STATUS,
    },
    rtps_udp_transport::RtpsUdpTransport,
    security::plugins::{
        authentication::{Authentication, ValidateLocalIdentityOut, ValidationResult},
        types::{DdsSecurityPlugins, SecurityException},
    },
    transport::types::Guid,
};

use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[test]
fn validate_local_identity_returns_error() {
    struct MockAuthentication;
    impl Authentication for MockAuthentication {
        type IdentityHandle = ();

        fn validate_local_identity(
            &mut self,
            _domain_id: DomainId,
            _participant_qos: &DomainParticipantQos,
            _candidate_participant_guid: Guid,
        ) -> Result<ValidateLocalIdentityOut<Self::IdentityHandle>, ValidationResult> {
            Err(ValidationResult::ValidationFailed(
                SecurityException::default(),
            ))
        }
    }
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let security_plugins = DdsSecurityPlugins {
        access_control_plugin: None::<()>,
        authentication_plugin: Some(MockAuthentication),
    };

    let participant_factory = DomainParticipantFactory::get_custom_instance(
        DustDdsConfiguration::default(),
        RtpsUdpTransport::default(),
        security_plugins,
    );
    assert!(
        participant_factory
            .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
            .is_err()
    );
}
