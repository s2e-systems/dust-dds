mod utils;

use dust_dds::{
    dds_async::domain_participant_factory::DomainParticipantFactoryAsync,
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        configuration::DustDdsConfiguration, listener::NO_LISTENER, qos::QosKind, status::NO_STATUS,
    },
    rtps_udp_transport::RtpsUdpTransport,
    security::{
        plugins::{
            access_control::AccessControl,
            authentication::{
                Authentication, BeginHandshakeReplyOut, BeginHandshakeRequestOut,
                ValidateRemoteIdentityOut, ValidationResult,
            },
            types::{DdsSecurityPlugins, SecurityException},
        },
        types::{
            AuthRequestMessageToken, AuthenticatedPeerCredentialToken, HandshakeMessageToken,
            IdentityStatusToken, IdentityToken, ParticipantSecurityConfig,
        },
    },
    std_runtime::StdRuntime,
    transport::types::Guid,
};

use crate::utils::{
    domain_id_generator::TEST_DOMAIN_ID_GENERATOR,
    security_stubs::{StubAccessControl, StubAuthentication},
};

fn create_test_participant_factory<Auth: Authentication, Access: AccessControl>(
    security_plugins: DdsSecurityPlugins<Auth, Access>,
) -> DomainParticipantFactory<RtpsUdpTransport> {
    let factory_async = Box::leak(Box::new(DomainParticipantFactoryAsync::new(
        [1, 2, 3, 4],
        [5, 6, 7, 8],
        DustDdsConfiguration::default(),
        StdRuntime::default(),
        RtpsUdpTransport::default(),
        security_plugins,
    )));
    DomainParticipantFactory::new(factory_async)
}

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
        access_control_plugin: None::<()>,
        authentication_plugin: Some(auth),
    };

    let participant_factory = create_test_participant_factory(security_plugins);
    assert!(
        participant_factory
            .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
            .is_err()
    );
}

#[test]
fn create_participant_when_validate_local_permissions_returns_error_should_fail() {
    let access = StubAccessControl {
        validate_local_permissions_fn: Some(Box::new(|_, _| Err(SecurityException::default()))),
        ..Default::default()
    };

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let security_plugins = DdsSecurityPlugins {
        access_control_plugin: Some(access),
        authentication_plugin: Some(StubAuthentication::default()),
    };

    let participant_factory = create_test_participant_factory(security_plugins);
    assert!(
        participant_factory
            .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
            .is_err()
    );
}

#[test]
fn create_participant_when_check_create_participant_returns_error_should_fail() {
    let access = StubAccessControl {
        check_create_participant_fn: Some(Box::new(|_, _, _| Err(SecurityException::default()))),
        ..Default::default()
    };

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let security_plugins = DdsSecurityPlugins {
        access_control_plugin: Some(access),
        authentication_plugin: Some(StubAuthentication::default()),
    };

    let participant_factory = create_test_participant_factory(security_plugins);
    assert!(
        participant_factory
            .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
            .is_err()
    );
}

#[test]
fn get_identity_token_returns_expected_token() {
    let mut auth = StubAuthentication {
        get_identity_token_fn: Some(Box::new(|handle| {
            if *handle == 42 {
                Ok(IdentityToken::default())
            } else {
                Err(SecurityException {
                    message: "Invalid handle".into(),
                    code: 1,
                    minor_code: 0,
                })
            }
        })),
        ..Default::default()
    };

    assert!(auth.get_identity_token(&42).is_ok());
    assert!(auth.get_identity_token(&99).is_err());
}

#[test]
fn validate_remote_identity_returns_expected_result() {
    let mut auth = StubAuthentication {
        validate_remote_identity_fn: Some(Box::new(|local_handle, _, _, _| {
            if *local_handle == 1 {
                Ok(ValidateRemoteIdentityOut {
                    remote_identity_handle: 100,
                    local_auth_request_token: AuthRequestMessageToken::default(),
                })
            } else {
                Err(ValidationResult::ValidationFailed(
                    SecurityException::default(),
                ))
            }
        })),
        ..Default::default()
    };

    let res =
        auth.validate_remote_identity(&1, IdentityToken::default(), None, Guid::from([2; 16]));
    assert!(res.is_ok());
    assert_eq!(res.unwrap().remote_identity_handle, 100);

    let res_err =
        auth.validate_remote_identity(&99, IdentityToken::default(), None, Guid::from([2; 16]));
    assert!(res_err.is_err());
}

#[test]
fn begin_handshake_request_returns_expected_result() {
    let mut auth = StubAuthentication {
        begin_handshake_request_fn: Some(Box::new(|initiator, replier, _| {
            if *initiator == 1 && *replier == 2 {
                Ok(BeginHandshakeRequestOut {
                    handshake_handle: 777,
                    handshake_message_token: HandshakeMessageToken::default(),
                })
            } else {
                Err(ValidationResult::ValidationFailed(
                    SecurityException::default(),
                ))
            }
        })),
        ..Default::default()
    };

    let res = auth.begin_handshake_request(&1, &2, &[]);
    assert!(res.is_ok());
    assert_eq!(res.unwrap().handshake_handle, 777);

    let res_err = auth.begin_handshake_request(&1, &99, &[]);
    assert!(res_err.is_err());
}

#[test]
fn begin_handshake_reply_returns_expected_result() {
    let mut auth = StubAuthentication {
        begin_handshake_reply_fn: Some(Box::new(|_, initiator, replier, _| {
            if *initiator == 10 && *replier == 20 {
                Ok(BeginHandshakeReplyOut {
                    handshake_handle: 888,
                    handshake_message_out: HandshakeMessageToken::default(),
                })
            } else {
                Err(ValidationResult::ValidationFailed(
                    SecurityException::default(),
                ))
            }
        })),
        ..Default::default()
    };

    let res = auth.begin_handshake_reply(HandshakeMessageToken::default(), &10, &20, &[]);
    assert!(res.is_ok());
    assert_eq!(res.unwrap().handshake_handle, 888);

    let res_err = auth.begin_handshake_reply(HandshakeMessageToken::default(), &10, &99, &[]);
    assert!(res_err.is_err());
}

#[test]
fn process_handshake_returns_expected_result() {
    let mut auth = StubAuthentication {
        process_handshake_fn: Some(Box::new(|_, handle| {
            if *handle == 777 {
                Ok(HandshakeMessageToken::default())
            } else {
                Err(ValidationResult::ValidationFailed(
                    SecurityException::default(),
                ))
            }
        })),
        ..Default::default()
    };

    let res = auth.process_handshake(HandshakeMessageToken::default(), &777);
    assert!(res.is_ok());

    let res_err = auth.process_handshake(HandshakeMessageToken::default(), &999);
    assert!(res_err.is_err());
}

#[test]
fn get_shared_secret_returns_expected_result() {
    let mut auth = StubAuthentication {
        get_shared_secret_fn: Some(Box::new(|handle| {
            if *handle == 12345 {
                Ok(99999)
            } else {
                Err(SecurityException {
                    message: "Invalid handshake handle".into(),
                    code: 1,
                    minor_code: 0,
                })
            }
        })),
        ..Default::default()
    };

    let res = auth.get_shared_secret(&12345);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 99999);

    let res_err = auth.get_shared_secret(&111);
    assert!(res_err.is_err());
}

#[test]
fn get_authenticated_peer_credential_token_returns_expected_result() {
    let mut auth = StubAuthentication {
        get_authenticated_peer_credential_token_fn: Some(Box::new(|handle| {
            if *handle == 5555 {
                Ok(AuthenticatedPeerCredentialToken::default())
            } else {
                Err(SecurityException {
                    message: "Invalid handshake handle".into(),
                    code: 1,
                    minor_code: 0,
                })
            }
        })),
        ..Default::default()
    };

    let res = auth.get_authenticated_peer_credential_token(&5555);
    assert!(res.is_ok());

    let res_err = auth.get_authenticated_peer_credential_token(&111);
    assert!(res_err.is_err());
}

#[test]
fn get_identity_status_token_returns_expected_token() {
    let mut auth = StubAuthentication {
        get_identity_status_token_fn: Some(Box::new(|handle| {
            if *handle == 100 {
                Ok(IdentityStatusToken::default())
            } else {
                Err(SecurityException {
                    message: "Invalid handle".into(),
                    code: 1,
                    minor_code: 0,
                })
            }
        })),
        ..Default::default()
    };

    let res = auth.get_identity_status_token(&100);
    assert!(res.is_ok());

    let res_err = auth.get_identity_status_token(&999);
    assert!(res_err.is_err());
}

#[test]
fn set_participant_security_config_returns_expected_result() {
    let mut auth = StubAuthentication {
        set_participant_security_config_fn: Some(Box::new(|handle, config| {
            if *handle == 10 {
                Ok(config.algorithm_info)
            } else {
                Err(SecurityException {
                    message: "Invalid handle".into(),
                    code: 1,
                    minor_code: 0,
                })
            }
        })),
        ..Default::default()
    };

    let config = ParticipantSecurityConfig::default();
    let res = auth.set_participant_security_config(&10, &config);
    assert!(res.is_ok());

    let res_err = auth.set_participant_security_config(&999, &config);
    assert!(res_err.is_err());
}
