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
        access_control::AccessControl,
        authentication::Authentication,
        types::{DdsSecurityPlugins, SecurityException},
    },
};

use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

#[test]
fn validate_local_identity_returns_error() {
    struct MockAccessControl;
    impl AccessControl for MockAccessControl {
        type IdentityHandle = ();
        type PermissionsHandle = ();

        fn validate_local_permissions(
            &mut self,
            _auth_plugin: &dyn Authentication<IdentityHandle = Self::IdentityHandle>,
            _identity: Self::IdentityHandle,
            _domain_id: DomainId,
            _participant_qos: &DomainParticipantQos,
        ) -> Result<Self::PermissionsHandle, SecurityException> {
            Err(SecurityException {
                ..Default::default()
            })
        }
    }
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let security_plugins = DdsSecurityPlugins {
        access_control_plugin: Some(MockAccessControl),
        authentication_plugin: None::<()>,
    };

    let participant_factory = DomainParticipantFactory::get_custom_instance(
        RtpsUdpTransport::default(),
        DustDdsConfiguration::default(),
        security_plugins,
    );
    assert!(
        participant_factory
            .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
            .is_err()
    );
}
