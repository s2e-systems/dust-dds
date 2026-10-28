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

fn create_custom_factory<Auth, Access, Crypto>(
    security: Option<DdsSecurityPlugins<Auth, Access, Crypto>>,
) -> DomainParticipantFactory
where
    Auth: dust_dds::security::plugins::authentication::Authentication,
    Access: dust_dds::security::plugins::access_control::AccessControl,
    Crypto: dust_dds::security::plugins::cryptographic::Cryptographic,
{
    let participant_factory_async = Box::leak(Box::new(
        dust_dds::dds_async::domain_participant_factory::DomainParticipantFactoryAsync::new(
            Default::default(),
            dust_dds::std_runtime::StdRuntime::default(),
            dust_dds::rtps_udp_transport::udp_transport::RtpsUdpTransport::default(),
            security,
        ),
    ));
    DomainParticipantFactory::new(participant_factory_async)
}

#[test]
fn when_allow_unauthenticated_false_non_secure_participant_is_rejected() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let sec_config = ParticipantSecurityConfig {
        allow_unauthenticated_participants: false,
        ..Default::default()
    };
    let access = StubAccessControl {
        get_participant_security_config_fn: Some(Box::new(move |_| Ok(sec_config.clone()))),
        ..Default::default()
    };
    let auth = StubAuthentication {
        validate_local_identity_fn: Some(Box::new(|_, _, candidate_guid| {
            let mut prefix = candidate_guid.prefix();
            prefix[11] = 1;
            Ok(
                dust_dds::security::plugins::authentication::ValidateLocalIdentityOut {
                    local_identity_handle: 1,
                    adjusted_participant_guid: dust_dds::transport::types::Guid::new(
                        prefix,
                        candidate_guid.entity_id(),
                    ),
                },
            )
        })),
        ..Default::default()
    };
    let security_plugins = DdsSecurityPlugins {
        access_control_plugin: access,
        authentication_plugin: auth,
        cryptographic_plugin: (),
    };
    let secure_factory = create_custom_factory(Some(security_plugins));
    let non_secure_factory = create_custom_factory(None::<DdsSecurityPlugins<(), (), ()>>);

    let secure_participant = secure_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let non_secure_participant = non_secure_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    // Wait until non-secure participant has received the secure participant's announcement
    let start = Instant::now();
    loop {
        if non_secure_participant
            .get_discovered_participants()
            .unwrap()
            .len()
            == 2
        {
            break;
        }
        if start.elapsed() > std::time::Duration::from_secs(10) {
            panic!("Non-secure participant did not discover secure participant within timeout");
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Give a short grace period for secure participant to also have processed non-secure participant's announcement
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Secure participant with allow_unauthenticated_participants=false must reject non-secure participant
    let secure_discovered = secure_participant.get_discovered_participants().unwrap();
    assert_eq!(
        secure_discovered.len(),
        1,
        "Secure participant should only see itself"
    );
    assert_eq!(
        secure_discovered[0].as_ref(),
        secure_participant.get_instance_handle().as_ref()
    );

    // Non-secure participant has no security enabled so it accepts the secure participant
    let non_secure_discovered = non_secure_participant
        .get_discovered_participants()
        .unwrap();
    assert_eq!(
        non_secure_discovered.len(),
        2,
        "Non-secure participant should see both"
    );
}

#[test]
fn when_allow_unauthenticated_false_incompatible_plugins_participant_is_rejected() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    // Participant 1: Plugin A
    let sec_config1 = ParticipantSecurityConfig {
        allow_unauthenticated_participants: false,
        ..Default::default()
    };
    let access1 = StubAccessControl {
        get_participant_security_config_fn: Some(Box::new(move |_| Ok(sec_config1.clone()))),
        ..Default::default()
    };
    let auth1 = StubAuthentication {
        validate_local_identity_fn: Some(Box::new(|_, _, candidate_guid| {
            let mut prefix = candidate_guid.prefix();
            prefix[11] = 1;
            Ok(
                dust_dds::security::plugins::authentication::ValidateLocalIdentityOut {
                    local_identity_handle: 1,
                    adjusted_participant_guid: dust_dds::transport::types::Guid::new(
                        prefix,
                        candidate_guid.entity_id(),
                    ),
                },
            )
        })),
        get_identity_token_fn: Some(Box::new(|_| {
            Ok(IdentityToken(DataHolder {
                class_id: "PluginA:1.0".to_string(),
                ..Default::default()
            }))
        })),
        ..Default::default()
    };
    let factory1 = create_custom_factory(Some(DdsSecurityPlugins {
        access_control_plugin: access1,
        authentication_plugin: auth1,
        cryptographic_plugin: (),
    }));

    // Participant 2: Plugin B (incompatible class_id)
    let sec_config2 = ParticipantSecurityConfig {
        allow_unauthenticated_participants: false,
        ..Default::default()
    };
    let access2 = StubAccessControl {
        get_participant_security_config_fn: Some(Box::new(move |_| Ok(sec_config2.clone()))),
        ..Default::default()
    };
    let auth2 = StubAuthentication {
        validate_local_identity_fn: Some(Box::new(|_, _, candidate_guid| {
            let mut prefix = candidate_guid.prefix();
            prefix[11] = 2;
            Ok(
                dust_dds::security::plugins::authentication::ValidateLocalIdentityOut {
                    local_identity_handle: 2,
                    adjusted_participant_guid: dust_dds::transport::types::Guid::new(
                        prefix,
                        candidate_guid.entity_id(),
                    ),
                },
            )
        })),
        get_identity_token_fn: Some(Box::new(|_| {
            Ok(IdentityToken(DataHolder {
                class_id: "PluginB:1.0".to_string(),
                ..Default::default()
            }))
        })),
        ..Default::default()
    };
    let factory2 = create_custom_factory(Some(DdsSecurityPlugins {
        access_control_plugin: access2,
        authentication_plugin: auth2,
        cryptographic_plugin: (),
    }));

    let participant1 = factory1
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let participant2 = factory2
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    std::thread::sleep(std::time::Duration::from_millis(500));

    // Both should reject each other because plugins are incompatible and allow_unauthenticated is false
    assert_eq!(participant1.get_discovered_participants().unwrap().len(), 1);
    assert_eq!(participant2.get_discovered_participants().unwrap().len(), 1);
}

#[test]
fn when_allow_unauthenticated_false_compatible_participant_is_accepted() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    // Participant 1
    let sec_config1 = ParticipantSecurityConfig {
        allow_unauthenticated_participants: false,
        ..Default::default()
    };
    let access1 = StubAccessControl {
        get_participant_security_config_fn: Some(Box::new(move |_| Ok(sec_config1.clone()))),
        ..Default::default()
    };
    let auth1 = StubAuthentication {
        validate_local_identity_fn: Some(Box::new(|_, _, candidate_guid| {
            let mut prefix = candidate_guid.prefix();
            prefix[11] = 1;
            Ok(
                dust_dds::security::plugins::authentication::ValidateLocalIdentityOut {
                    local_identity_handle: 1,
                    adjusted_participant_guid: dust_dds::transport::types::Guid::new(
                        prefix,
                        candidate_guid.entity_id(),
                    ),
                },
            )
        })),
        get_identity_token_fn: Some(Box::new(|_| {
            Ok(IdentityToken(DataHolder {
                class_id: "PluginA:1.0".to_string(),
                ..Default::default()
            }))
        })),
        ..Default::default()
    };
    let factory1 = create_custom_factory(Some(DdsSecurityPlugins {
        access_control_plugin: access1,
        authentication_plugin: auth1,
        cryptographic_plugin: (),
    }));

    // Participant 2 (compatible plugin class_id)
    let sec_config2 = ParticipantSecurityConfig {
        allow_unauthenticated_participants: false,
        ..Default::default()
    };
    let access2 = StubAccessControl {
        get_participant_security_config_fn: Some(Box::new(move |_| Ok(sec_config2.clone()))),
        ..Default::default()
    };
    let auth2 = StubAuthentication {
        validate_local_identity_fn: Some(Box::new(|_, _, candidate_guid| {
            let mut prefix = candidate_guid.prefix();
            prefix[11] = 2;
            Ok(
                dust_dds::security::plugins::authentication::ValidateLocalIdentityOut {
                    local_identity_handle: 2,
                    adjusted_participant_guid: dust_dds::transport::types::Guid::new(
                        prefix,
                        candidate_guid.entity_id(),
                    ),
                },
            )
        })),
        get_identity_token_fn: Some(Box::new(|_| {
            Ok(IdentityToken(DataHolder {
                class_id: "PluginA:1.0".to_string(),
                ..Default::default()
            }))
        })),
        ..Default::default()
    };
    let factory2 = create_custom_factory(Some(DdsSecurityPlugins {
        access_control_plugin: access2,
        authentication_plugin: auth2,
        cryptographic_plugin: (),
    }));

    let participant1 = factory1
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let participant2 = factory2
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
}

#[test]
fn when_allow_unauthenticated_true_non_secure_participant_is_accepted() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let sec_config = ParticipantSecurityConfig {
        allow_unauthenticated_participants: true,
        ..Default::default()
    };
    let access = StubAccessControl {
        get_participant_security_config_fn: Some(Box::new(move |_| Ok(sec_config.clone()))),
        ..Default::default()
    };
    let auth = StubAuthentication {
        validate_local_identity_fn: Some(Box::new(|_, _, candidate_guid| {
            let mut prefix = candidate_guid.prefix();
            prefix[11] = 1;
            Ok(
                dust_dds::security::plugins::authentication::ValidateLocalIdentityOut {
                    local_identity_handle: 1,
                    adjusted_participant_guid: dust_dds::transport::types::Guid::new(
                        prefix,
                        candidate_guid.entity_id(),
                    ),
                },
            )
        })),
        ..Default::default()
    };
    let security_plugins = DdsSecurityPlugins {
        access_control_plugin: access,
        authentication_plugin: auth,
        cryptographic_plugin: (),
    };
    let secure_factory = create_custom_factory(Some(security_plugins));
    let non_secure_factory = create_custom_factory(None::<DdsSecurityPlugins<(), (), ()>>);

    let secure_participant = secure_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let non_secure_participant = non_secure_factory
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let start = Instant::now();
    loop {
        if secure_participant
            .get_discovered_participants()
            .unwrap()
            .len()
            == 2
            && non_secure_participant
                .get_discovered_participants()
                .unwrap()
                .len()
                == 2
        {
            break;
        }
        if start.elapsed() > std::time::Duration::from_secs(10) {
            panic!("Participants not discovered within timeout");
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    assert_eq!(
        secure_participant
            .get_discovered_participants()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        non_secure_participant
            .get_discovered_participants()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn when_allow_unauthenticated_true_incompatible_plugins_participant_is_accepted() {
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();

    let sec_config1 = ParticipantSecurityConfig {
        allow_unauthenticated_participants: true,
        ..Default::default()
    };
    let access1 = StubAccessControl {
        get_participant_security_config_fn: Some(Box::new(move |_| Ok(sec_config1.clone()))),
        ..Default::default()
    };
    let auth1 = StubAuthentication {
        validate_local_identity_fn: Some(Box::new(|_, _, candidate_guid| {
            let mut prefix = candidate_guid.prefix();
            prefix[11] = 1;
            Ok(
                dust_dds::security::plugins::authentication::ValidateLocalIdentityOut {
                    local_identity_handle: 1,
                    adjusted_participant_guid: dust_dds::transport::types::Guid::new(
                        prefix,
                        candidate_guid.entity_id(),
                    ),
                },
            )
        })),
        get_identity_token_fn: Some(Box::new(|_| {
            Ok(IdentityToken(DataHolder {
                class_id: "PluginA:1.0".to_string(),
                ..Default::default()
            }))
        })),
        ..Default::default()
    };
    let factory1 = create_custom_factory(Some(DdsSecurityPlugins {
        access_control_plugin: access1,
        authentication_plugin: auth1,
        cryptographic_plugin: (),
    }));

    let sec_config2 = ParticipantSecurityConfig {
        allow_unauthenticated_participants: true,
        ..Default::default()
    };
    let access2 = StubAccessControl {
        get_participant_security_config_fn: Some(Box::new(move |_| Ok(sec_config2.clone()))),
        ..Default::default()
    };
    let auth2 = StubAuthentication {
        validate_local_identity_fn: Some(Box::new(|_, _, candidate_guid| {
            let mut prefix = candidate_guid.prefix();
            prefix[11] = 2;
            Ok(
                dust_dds::security::plugins::authentication::ValidateLocalIdentityOut {
                    local_identity_handle: 2,
                    adjusted_participant_guid: dust_dds::transport::types::Guid::new(
                        prefix,
                        candidate_guid.entity_id(),
                    ),
                },
            )
        })),
        get_identity_token_fn: Some(Box::new(|_| {
            Ok(IdentityToken(DataHolder {
                class_id: "PluginB:1.0".to_string(),
                ..Default::default()
            }))
        })),
        ..Default::default()
    };
    let factory2 = create_custom_factory(Some(DdsSecurityPlugins {
        access_control_plugin: access2,
        authentication_plugin: auth2,
        cryptographic_plugin: (),
    }));

    let participant1 = factory1
        .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
        .unwrap();

    let participant2 = factory2
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
}
