mod utils;

use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{listener::NO_LISTENER, qos::QosKind, status::NO_STATUS},
    security::plugins::{
        authentication::ValidationResult,
        types::{DdsSecurityPlugins, SecurityException},
    },
};

use crate::utils::{
    domain_id_generator::TEST_DOMAIN_ID_GENERATOR,
    security_stubs::{StubAccessControl, StubAuthentication},
};

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
        cryptographic_plugin: None::<()>,
    };

    let participant_factory = DomainParticipantFactory::get_custom_instance(
        Default::default(),
        Default::default(),
        security_plugins,
    );
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
        cryptographic_plugin: None::<()>,
    };

    let participant_factory = DomainParticipantFactory::get_custom_instance(
        Default::default(),
        Default::default(),
        security_plugins,
    );
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
        cryptographic_plugin: None::<()>,
    };

    let participant_factory = DomainParticipantFactory::get_custom_instance(
        Default::default(),
        Default::default(),
        security_plugins,
    );
    assert!(
        participant_factory
            .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
            .is_err()
    );
}
