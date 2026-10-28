mod domain_id_generator;
mod security_stubs;

use std::time::Instant;

use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{error::DdsError, listener::NO_LISTENER, qos::QosKind, status::NO_STATUS},
    security::{
        plugins::{
            authentication::ValidationResult,
            types::{DdsSecurityPlugins, SecurityException},
        },
        types::{
            CryptoAlgorithmRequirements, DataHolder, IdentityToken,
            ParticipantSecurityAlgorithmInfo, ParticipantSecurityConfig,
            ParticipantSecurityDigitalSignatureAlgorithmInfo,
            ParticipantSecurityKeyEstablishmentAlgorithmInfo,
            ParticipantSecuritySymmetricCipherAlgorithmInfo, PermissionsToken,
        },
    },
};

use domain_id_generator::TEST_DOMAIN_ID_GENERATOR;
use security_stubs::{StubAccessControl, StubAuthentication};

#[test]
fn create_participant_when_validate_local_identity_returns_error_should_fail() {
    let auth = StubAuthentication {
        validate_local_identity_fn: Some(Box::new(|_, _, _| {
            Err(ValidationResult::ValidationFailed(
                SecurityException::default(),
            ))
        })),
        ..Default::default()
    };

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let security_plugins = DdsSecurityPlugins {
        access_control_plugin: (),
        authentication_plugin: auth,
        cryptographic_plugin: (),
    };

    let participant_factory = DomainParticipantFactory::get_custom_instance(
        Default::default(),
        Default::default(),
        Some(security_plugins),
    );
    assert!(matches!(
        participant_factory.create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS),
        Err(DdsError::NotAllowedBySecurity)
    ));
}

#[test]
fn create_participant_when_validate_local_permissions_returns_error_should_fail() {
    let access = StubAccessControl {
        validate_local_permissions_fn: Some(Box::new(|_, _| Err(SecurityException::default()))),
        ..Default::default()
    };

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let security_plugins = DdsSecurityPlugins {
        access_control_plugin: access,
        authentication_plugin: StubAuthentication::default(),
        cryptographic_plugin: (),
    };

    let participant_factory = DomainParticipantFactory::get_custom_instance(
        Default::default(),
        Default::default(),
        Some(security_plugins),
    );
    assert!(matches!(
        participant_factory.create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS),
        Err(DdsError::NotAllowedBySecurity)
    ));
}

#[test]
fn create_participant_when_check_create_participant_returns_error_should_fail() {
    let access = StubAccessControl {
        check_create_participant_fn: Some(Box::new(|_, _, _| Err(SecurityException::default()))),
        ..Default::default()
    };

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let security_plugins = DdsSecurityPlugins {
        access_control_plugin: access,
        authentication_plugin: StubAuthentication::default(),
        cryptographic_plugin: (),
    };

    let participant_factory = DomainParticipantFactory::get_custom_instance(
        Default::default(),
        Default::default(),
        Some(security_plugins),
    );
    assert!(matches!(
        participant_factory.create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS),
        Err(DdsError::NotAllowedBySecurity)
    ));
}

#[test]
fn participant_propagates_security_data_on_discovery() {
    let expected_identity_token = IdentityToken(DataHolder {
        class_id: "mock_identity_token".to_string(),
        ..Default::default()
    });
    let expected_permissions_token = PermissionsToken(DataHolder {
        class_id: "mock_permissions_token".to_string(),
        ..Default::default()
    });
    let expected_security_config = ParticipantSecurityConfig {
        is_rtps_axk_protected: true,
        is_discovery_protected: true,
        is_liveliness_protected: false,
        is_key_revision_enabled: true,
        is_rtps_psk_protected: false,
        allow_unauthenticated_participants: true,
        is_access_protected: true,
        plugin_participant_attributes: 1234,
        ac_endpoint_properties: Default::default(),
        algorithm_info: ParticipantSecurityAlgorithmInfo {
            digital_signature: ParticipantSecurityDigitalSignatureAlgorithmInfo {
                trust_chain: CryptoAlgorithmRequirements {
                    supported_mask: 1,
                    required_mask: 1,
                },
                message_auth: CryptoAlgorithmRequirements {
                    supported_mask: 2,
                    required_mask: 2,
                },
            },
            key_establishment: ParticipantSecurityKeyEstablishmentAlgorithmInfo {
                shared_secret: CryptoAlgorithmRequirements {
                    supported_mask: 3,
                    required_mask: 3,
                },
            },
            symmetric_cipher: ParticipantSecuritySymmetricCipherAlgorithmInfo {
                supported_mask: 4,
                builtin_endpoints_required_mask: 4,
                builtin_kx_endpoints_required_mask: 4,
                user_endpoints_default_required_mask: 4,
            },
        },
    };

    let id_token = expected_identity_token.clone();
    let auth = StubAuthentication {
        get_identity_token_fn: Some(Box::new(move |_| Ok(id_token.clone()))),
        ..Default::default()
    };

    let perm_token = expected_permissions_token.clone();
    let sec_config = expected_security_config.clone();
    let access = StubAccessControl {
        get_permissions_token_fn: Some(Box::new(move |_| Ok(perm_token.clone()))),
        get_participant_security_config_fn: Some(Box::new(move |_| Ok(sec_config.clone()))),
        ..Default::default()
    };

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let security_plugins = DdsSecurityPlugins {
        access_control_plugin: access,
        authentication_plugin: auth,
        cryptographic_plugin: (),
    };

    let participant_factory_async = Box::leak(Box::new(
        dust_dds::dds_async::domain_participant_factory::DomainParticipantFactoryAsync::new(
            Default::default(),
            dust_dds::std_runtime::StdRuntime::default(),
            dust_dds::rtps_udp_transport::udp_transport::RtpsUdpTransport::default(),
            Some(security_plugins),
        ),
    ));
    let participant_factory = DomainParticipantFactory::new(participant_factory_async);

    let participant1 = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let participant2 = participant_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let start = Instant::now();
    loop {
        if participant1.get_discovered_participants().unwrap().len() == 2
            && participant2.get_discovered_participants().unwrap().len() == 2
        {
            break;
        }
        if start.elapsed() > std::time::Duration::from_secs(10) {
            panic!("Participants not discovered within timeout");
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    assert_eq!(participant1.get_discovered_participants().unwrap().len(), 2);
    assert_eq!(participant2.get_discovered_participants().unwrap().len(), 2);

    let discovered = participant2.get_discovered_participants().unwrap();
    let remote_handle = discovered
        .into_iter()
        .find(|h| h.as_ref() == participant1.get_instance_handle().as_ref())
        .expect("Participant 1 must be in discovered list of participant 2");
    let participant_data = participant2
        .get_discovered_participant_data(remote_handle)
        .unwrap();

    assert_eq!(
        participant_data.key().value,
        *participant1.get_instance_handle().as_ref()
    );
    assert_eq!(
        participant_data.identity_token(),
        &Some(expected_identity_token)
    );
    assert_eq!(
        participant_data.permissions_token(),
        &Some(expected_permissions_token)
    );
    assert_eq!(
        participant_data.protection_info(),
        &Some((&expected_security_config).into())
    );
    assert_eq!(
        participant_data.digital_signature(),
        &Some(expected_security_config.algorithm_info.digital_signature)
    );
    assert_eq!(
        participant_data.key_establishment(),
        &Some(expected_security_config.algorithm_info.key_establishment)
    );
    assert_eq!(
        participant_data.symmetric_cipher(),
        &Some(expected_security_config.algorithm_info.symmetric_cipher)
    );
}
