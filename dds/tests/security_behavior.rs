mod utils;
use dust_dds::{
    builtin_topics::{
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    dds_async::domain_participant_factory::DomainParticipantFactoryAsync,
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{
        configuration::DustDdsConfiguration,
        domain::DomainId,
        instance::InstanceHandle,
        listener::NO_LISTENER,
        qos::{DataReaderQos, DataWriterQos, DomainParticipantQos, QosKind, TopicQos},
        qos_policy::{DataTagQosPolicy, PartitionQosPolicy},
        status::NO_STATUS,
    },
    rtps_udp_transport::RtpsUdpTransport,
    security::{
        plugins::{
            access_control::{AccessControl, CheckRemoteDataReaderOut},
            access_control_listener::AccessControlListener,
            authentication::{
                Authentication, BeginHandshakeReplyOut, BeginHandshakeRequestOut,
                ValidateLocalIdentityOut, ValidateRemoteIdentityOut, ValidationResult,
            },
            authentication_listener::AuthenticationListener,
            types::{DdsSecurityPlugins, SecurityException},
        },
        types::{
            AuthRequestMessageToken, AuthenticatedPeerCredentialToken, HandshakeMessageToken,
            IdentityStatusToken, IdentityToken, ParticipantSecurityAlgorithmInfo,
            ParticipantSecurityConfig, PermissionsCredentialToken, PermissionsToken,
        },
    },
    std_runtime::StdRuntime,
    transport::types::Guid,
};

use crate::utils::domain_id_generator::TEST_DOMAIN_ID_GENERATOR;

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
    struct MockAuthentication;
    impl Authentication for MockAuthentication {
        type IdentityHandle = ();
        type HandshakeHandle = ();
        type SharedSecretHandle = ();

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

        fn validate_remote_identity(
            &mut self,
            _local_identity_handle: &Self::IdentityHandle,
            _remote_identity_token: IdentityToken,
            _remote_auth_request_token: Option<AuthRequestMessageToken>,
            _remote_participant_guid: Guid,
        ) -> Result<ValidateRemoteIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_request(
            &mut self,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeRequestOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_reply(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeReplyOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn process_handshake(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<HandshakeMessageToken, ValidationResult> {
            unimplemented!()
        }

        fn get_shared_secret(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<Self::SharedSecretHandle, SecurityException> {
            unimplemented!()
        }

        fn get_authenticated_peer_credential_token(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<AuthenticatedPeerCredentialToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_status_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityStatusToken, SecurityException> {
            unimplemented!()
        }

        fn set_participant_security_config(
            &mut self,
            _handle: &Self::IdentityHandle,
            _participant_security_config: &ParticipantSecurityConfig,
        ) -> Result<ParticipantSecurityAlgorithmInfo, SecurityException> {
            unimplemented!()
        }

        fn set_permissions_credential_and_token(
            &mut self,
            _handle: &Self::IdentityHandle,
            _permissions_credential_token: PermissionsCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn set_listener<L>(&mut self, _listener: Option<L>) -> Result<(), SecurityException>
        where
            L: AuthenticationListener<IdentityHandle = Self::IdentityHandle>,
        {
            unimplemented!()
        }

        fn return_identity_token(
            &mut self,
            _token: IdentityToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_status_token(
            &mut self,
            _token: IdentityStatusToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_authenticated_peer_credential_token(
            &mut self,
            _peer_credential_token: AuthenticatedPeerCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_handshake_handle(
            &mut self,
            _handshake_handle: Self::HandshakeHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_handle(
            &mut self,
            _identity_handle: Self::IdentityHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_sharedsecret_handle(
            &mut self,
            _sharedsecret_handle: Self::SharedSecretHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }
    }
    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let security_plugins = DdsSecurityPlugins {
        access_control_plugin: None::<()>,
        authentication_plugin: Some(MockAuthentication),
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
    struct MockAuthentication;
    impl Authentication for MockAuthentication {
        type IdentityHandle = ();
        type HandshakeHandle = ();
        type SharedSecretHandle = ();

        fn validate_local_identity(
            &mut self,
            _domain_id: DomainId,
            _participant_qos: &DomainParticipantQos,
            _candidate_participant_guid: Guid,
        ) -> Result<ValidateLocalIdentityOut<Self::IdentityHandle>, ValidationResult> {
            Ok(ValidateLocalIdentityOut {
                local_identity_handle: (),
                adjusted_participant_guid: Guid::from([1; 16]),
            })
        }

        fn validate_remote_identity(
            &mut self,
            _local_identity_handle: &Self::IdentityHandle,
            _remote_identity_token: IdentityToken,
            _remote_auth_request_token: Option<AuthRequestMessageToken>,
            _remote_participant_guid: Guid,
        ) -> Result<ValidateRemoteIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_request(
            &mut self,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeRequestOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_reply(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeReplyOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn process_handshake(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<HandshakeMessageToken, ValidationResult> {
            unimplemented!()
        }

        fn get_shared_secret(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<Self::SharedSecretHandle, SecurityException> {
            unimplemented!()
        }

        fn get_authenticated_peer_credential_token(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<AuthenticatedPeerCredentialToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_status_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityStatusToken, SecurityException> {
            unimplemented!()
        }

        fn set_participant_security_config(
            &mut self,
            _handle: &Self::IdentityHandle,
            _participant_security_config: &ParticipantSecurityConfig,
        ) -> Result<ParticipantSecurityAlgorithmInfo, SecurityException> {
            unimplemented!()
        }

        fn set_permissions_credential_and_token(
            &mut self,
            _handle: &Self::IdentityHandle,
            _permissions_credential_token: PermissionsCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn set_listener<L>(&mut self, _listener: Option<L>) -> Result<(), SecurityException>
        where
            L: AuthenticationListener<IdentityHandle = Self::IdentityHandle>,
        {
            unimplemented!()
        }

        fn return_identity_token(
            &mut self,
            _token: IdentityToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_status_token(
            &mut self,
            _token: IdentityStatusToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_authenticated_peer_credential_token(
            &mut self,
            _peer_credential_token: AuthenticatedPeerCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_handshake_handle(
            &mut self,
            _handshake_handle: Self::HandshakeHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_handle(
            &mut self,
            _identity_handle: Self::IdentityHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_sharedsecret_handle(
            &mut self,
            _sharedsecret_handle: Self::SharedSecretHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }
    }

    struct MockAccessControl;
    impl AccessControl for MockAccessControl {
        type PermissionsHandle = ();

        fn validate_local_permissions<A>(
            &mut self,
            _auth_plugin: &mut A,
            _identity: &A::IdentityHandle,
            _domain_id: DomainId,
            _participant_qos: &DomainParticipantQos,
        ) -> Result<Self::PermissionsHandle, SecurityException>
        where
            A: Authentication,
        {
            Err(SecurityException::default())
        }

        fn validate_remote_permissions<A>(
            &mut self,
            _auth_plugin: &mut A,
            _local_identity_handle: &A::IdentityHandle,
            _remote_identity_handle: &A::IdentityHandle,
            _remote_permissions_token: PermissionsToken,
            _remote_credential_token: AuthenticatedPeerCredentialToken,
        ) -> Result<Self::PermissionsHandle, SecurityException>
        where
            A: Authentication,
        {
            unimplemented!()
        }

        fn check_create_participant(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _domain_id: DomainId,
            _qos: &DomainParticipantQos,
        ) -> Result<(), SecurityException> {
            Ok(())
        }

        fn check_create_datawriter(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _domain_id: DomainId,
            _topic_name: &str,
            _qos: &DataWriterQos,
            _partition: &PartitionQosPolicy,
            _data_tag: &DataTagQosPolicy,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_create_datareader(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _domain_id: DomainId,
            _topic_name: &str,
            _qos: &DataReaderQos,
            _partition: &PartitionQosPolicy,
            _data_tag: &DataTagQosPolicy,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_create_topic(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _domain_id: DomainId,
            _topic_name: &str,
            _qos: &TopicQos,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_local_datawriter_register_instance(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _writer: &PublicationBuiltinTopicData,
            _key: &InstanceHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_local_datawriter_dispose_instance(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _writer: &PublicationBuiltinTopicData,
            _key: &InstanceHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_remote_participant(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _domain_id: DomainId,
            _participant_data: &ParticipantBuiltinTopicData,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_remote_datawriter(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _domain_id: DomainId,
            _publication_data: &PublicationBuiltinTopicData,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_remote_datareader(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _domain_id: DomainId,
            _subscription_data: &SubscriptionBuiltinTopicData,
        ) -> Result<CheckRemoteDataReaderOut, SecurityException> {
            unimplemented!()
        }

        fn check_remote_topic(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _domain_id: DomainId,
            _topic_data: &TopicBuiltinTopicData,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_local_datawriter_match(
            &mut self,
            _writer_permissions_handle: &Self::PermissionsHandle,
            _reader_permissions_handle: &Self::PermissionsHandle,
            _publication_data: &PublicationBuiltinTopicData,
            _subscription_data: &SubscriptionBuiltinTopicData,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_local_datareader_match(
            &mut self,
            _reader_permissions_handle: &Self::PermissionsHandle,
            _writer_permissions_handle: &Self::PermissionsHandle,
            _subscription_data: &SubscriptionBuiltinTopicData,
            _publication_data: &PublicationBuiltinTopicData,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_remote_datawriter_register_instance(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _reader: &SubscriptionBuiltinTopicData,
            _publication_handle: &InstanceHandle,
            _key: &InstanceHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_remote_datawriter_dispose_instance(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _reader: &SubscriptionBuiltinTopicData,
            _publication_handle: &InstanceHandle,
            _key: &InstanceHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn get_permissions_token(
            &mut self,
            _handle: &Self::PermissionsHandle,
        ) -> Result<PermissionsToken, SecurityException> {
            unimplemented!()
        }

        fn get_permissions_credential_token(
            &mut self,
            _handle: &Self::PermissionsHandle,
        ) -> Result<PermissionsCredentialToken, SecurityException> {
            unimplemented!()
        }

        fn set_listener<L>(&mut self, _listener: Option<L>) -> Result<(), SecurityException>
        where
            L: AccessControlListener<PermissionsHandle = Self::PermissionsHandle>,
        {
            unimplemented!()
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let security_plugins = DdsSecurityPlugins {
        access_control_plugin: Some(MockAccessControl),
        authentication_plugin: Some(MockAuthentication),
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
    struct MockAuthentication;
    impl Authentication for MockAuthentication {
        type IdentityHandle = ();
        type HandshakeHandle = ();
        type SharedSecretHandle = ();

        fn validate_local_identity(
            &mut self,
            _domain_id: DomainId,
            _participant_qos: &DomainParticipantQos,
            _candidate_participant_guid: Guid,
        ) -> Result<ValidateLocalIdentityOut<Self::IdentityHandle>, ValidationResult> {
            Ok(ValidateLocalIdentityOut {
                local_identity_handle: (),
                adjusted_participant_guid: Guid::from([1; 16]),
            })
        }

        fn validate_remote_identity(
            &mut self,
            _local_identity_handle: &Self::IdentityHandle,
            _remote_identity_token: IdentityToken,
            _remote_auth_request_token: Option<AuthRequestMessageToken>,
            _remote_participant_guid: Guid,
        ) -> Result<ValidateRemoteIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_request(
            &mut self,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeRequestOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_reply(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeReplyOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn process_handshake(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<HandshakeMessageToken, ValidationResult> {
            unimplemented!()
        }

        fn get_shared_secret(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<Self::SharedSecretHandle, SecurityException> {
            unimplemented!()
        }

        fn get_authenticated_peer_credential_token(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<AuthenticatedPeerCredentialToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_status_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityStatusToken, SecurityException> {
            unimplemented!()
        }

        fn set_participant_security_config(
            &mut self,
            _handle: &Self::IdentityHandle,
            _participant_security_config: &ParticipantSecurityConfig,
        ) -> Result<ParticipantSecurityAlgorithmInfo, SecurityException> {
            unimplemented!()
        }

        fn set_permissions_credential_and_token(
            &mut self,
            _handle: &Self::IdentityHandle,
            _permissions_credential_token: PermissionsCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn set_listener<L>(&mut self, _listener: Option<L>) -> Result<(), SecurityException>
        where
            L: AuthenticationListener<IdentityHandle = Self::IdentityHandle>,
        {
            unimplemented!()
        }

        fn return_identity_token(
            &mut self,
            _token: IdentityToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_status_token(
            &mut self,
            _token: IdentityStatusToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_authenticated_peer_credential_token(
            &mut self,
            _peer_credential_token: AuthenticatedPeerCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_handshake_handle(
            &mut self,
            _handshake_handle: Self::HandshakeHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_handle(
            &mut self,
            _identity_handle: Self::IdentityHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_sharedsecret_handle(
            &mut self,
            _sharedsecret_handle: Self::SharedSecretHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }
    }

    struct MockAccessControl;
    impl AccessControl for MockAccessControl {
        type PermissionsHandle = ();

        fn validate_local_permissions<A>(
            &mut self,
            _auth_plugin: &mut A,
            _identity: &A::IdentityHandle,
            _domain_id: DomainId,
            _participant_qos: &DomainParticipantQos,
        ) -> Result<Self::PermissionsHandle, SecurityException>
        where
            A: Authentication,
        {
            Ok(())
        }

        fn validate_remote_permissions<A>(
            &mut self,
            _auth_plugin: &mut A,
            _local_identity_handle: &A::IdentityHandle,
            _remote_identity_handle: &A::IdentityHandle,
            _remote_permissions_token: PermissionsToken,
            _remote_credential_token: AuthenticatedPeerCredentialToken,
        ) -> Result<Self::PermissionsHandle, SecurityException>
        where
            A: Authentication,
        {
            unimplemented!()
        }

        fn check_create_participant(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _domain_id: DomainId,
            _qos: &DomainParticipantQos,
        ) -> Result<(), SecurityException> {
            Err(SecurityException::default())
        }

        fn check_create_datawriter(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _domain_id: DomainId,
            _topic_name: &str,
            _qos: &DataWriterQos,
            _partition: &PartitionQosPolicy,
            _data_tag: &DataTagQosPolicy,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_create_datareader(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _domain_id: DomainId,
            _topic_name: &str,
            _qos: &DataReaderQos,
            _partition: &PartitionQosPolicy,
            _data_tag: &DataTagQosPolicy,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_create_topic(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _domain_id: DomainId,
            _topic_name: &str,
            _qos: &TopicQos,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_local_datawriter_register_instance(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _writer: &PublicationBuiltinTopicData,
            _key: &InstanceHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_local_datawriter_dispose_instance(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _writer: &PublicationBuiltinTopicData,
            _key: &InstanceHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_remote_participant(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _domain_id: DomainId,
            _participant_data: &ParticipantBuiltinTopicData,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_remote_datawriter(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _domain_id: DomainId,
            _publication_data: &PublicationBuiltinTopicData,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_remote_datareader(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _domain_id: DomainId,
            _subscription_data: &SubscriptionBuiltinTopicData,
        ) -> Result<CheckRemoteDataReaderOut, SecurityException> {
            unimplemented!()
        }

        fn check_remote_topic(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _domain_id: DomainId,
            _topic_data: &TopicBuiltinTopicData,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_local_datawriter_match(
            &mut self,
            _writer_permissions_handle: &Self::PermissionsHandle,
            _reader_permissions_handle: &Self::PermissionsHandle,
            _publication_data: &PublicationBuiltinTopicData,
            _subscription_data: &SubscriptionBuiltinTopicData,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_local_datareader_match(
            &mut self,
            _reader_permissions_handle: &Self::PermissionsHandle,
            _writer_permissions_handle: &Self::PermissionsHandle,
            _subscription_data: &SubscriptionBuiltinTopicData,
            _publication_data: &PublicationBuiltinTopicData,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_remote_datawriter_register_instance(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _reader: &SubscriptionBuiltinTopicData,
            _publication_handle: &InstanceHandle,
            _key: &InstanceHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn check_remote_datawriter_dispose_instance(
            &mut self,
            _permissions_handle: &Self::PermissionsHandle,
            _reader: &SubscriptionBuiltinTopicData,
            _publication_handle: &InstanceHandle,
            _key: &InstanceHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn get_permissions_token(
            &mut self,
            _handle: &Self::PermissionsHandle,
        ) -> Result<PermissionsToken, SecurityException> {
            unimplemented!()
        }

        fn get_permissions_credential_token(
            &mut self,
            _handle: &Self::PermissionsHandle,
        ) -> Result<PermissionsCredentialToken, SecurityException> {
            unimplemented!()
        }

        fn set_listener<L>(&mut self, _listener: Option<L>) -> Result<(), SecurityException>
        where
            L: AccessControlListener<PermissionsHandle = Self::PermissionsHandle>,
        {
            unimplemented!()
        }
    }

    let domain_id = TEST_DOMAIN_ID_GENERATOR.generate_unique_domain_id();
    let security_plugins = DdsSecurityPlugins {
        access_control_plugin: Some(MockAccessControl),
        authentication_plugin: Some(MockAuthentication),
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
    struct MockAuthentication;
    impl Authentication for MockAuthentication {
        type IdentityHandle = u32;
        type HandshakeHandle = ();
        type SharedSecretHandle = ();

        fn validate_local_identity(
            &mut self,
            _domain_id: DomainId,
            _participant_qos: &DomainParticipantQos,
            _candidate_participant_guid: Guid,
        ) -> Result<ValidateLocalIdentityOut<Self::IdentityHandle>, ValidationResult> {
            Ok(ValidateLocalIdentityOut {
                local_identity_handle: 42,
                adjusted_participant_guid: Guid::from([1; 16]),
            })
        }

        fn validate_remote_identity(
            &mut self,
            _local_identity_handle: &Self::IdentityHandle,
            _remote_identity_token: IdentityToken,
            _remote_auth_request_token: Option<AuthRequestMessageToken>,
            _remote_participant_guid: Guid,
        ) -> Result<ValidateRemoteIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_request(
            &mut self,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeRequestOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_reply(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeReplyOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn process_handshake(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<HandshakeMessageToken, ValidationResult> {
            unimplemented!()
        }

        fn get_shared_secret(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<Self::SharedSecretHandle, SecurityException> {
            unimplemented!()
        }

        fn get_authenticated_peer_credential_token(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<AuthenticatedPeerCredentialToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_token(
            &mut self,
            handle: &Self::IdentityHandle,
        ) -> Result<IdentityToken, SecurityException> {
            if *handle == 42 {
                Ok(IdentityToken::default())
            } else {
                Err(SecurityException {
                    message: "Invalid handle".into(),
                    code: 1,
                    minor_code: 0,
                })
            }
        }

        fn get_identity_status_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityStatusToken, SecurityException> {
            unimplemented!()
        }

        fn set_participant_security_config(
            &mut self,
            _handle: &Self::IdentityHandle,
            _participant_security_config: &ParticipantSecurityConfig,
        ) -> Result<ParticipantSecurityAlgorithmInfo, SecurityException> {
            unimplemented!()
        }

        fn set_permissions_credential_and_token(
            &mut self,
            _handle: &Self::IdentityHandle,
            _permissions_credential_token: PermissionsCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn set_listener<L>(&mut self, _listener: Option<L>) -> Result<(), SecurityException>
        where
            L: AuthenticationListener<IdentityHandle = Self::IdentityHandle>,
        {
            unimplemented!()
        }

        fn return_identity_token(
            &mut self,
            _token: IdentityToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_status_token(
            &mut self,
            _token: IdentityStatusToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_authenticated_peer_credential_token(
            &mut self,
            _peer_credential_token: AuthenticatedPeerCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_handshake_handle(
            &mut self,
            _handshake_handle: Self::HandshakeHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_handle(
            &mut self,
            _identity_handle: Self::IdentityHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_sharedsecret_handle(
            &mut self,
            _sharedsecret_handle: Self::SharedSecretHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }
    }

    let mut auth = MockAuthentication;
    assert!(auth.get_identity_token(&42).is_ok());
    assert!(auth.get_identity_token(&99).is_err());
}

#[test]
fn validate_remote_identity_returns_expected_result() {
    struct MockAuthentication;
    impl Authentication for MockAuthentication {
        type IdentityHandle = u32;
        type HandshakeHandle = ();
        type SharedSecretHandle = ();

        fn validate_local_identity(
            &mut self,
            _domain_id: DomainId,
            _participant_qos: &DomainParticipantQos,
            _candidate_participant_guid: Guid,
        ) -> Result<ValidateLocalIdentityOut<Self::IdentityHandle>, ValidationResult> {
            Ok(ValidateLocalIdentityOut {
                local_identity_handle: 1,
                adjusted_participant_guid: Guid::from([1; 16]),
            })
        }

        fn validate_remote_identity(
            &mut self,
            local_identity_handle: &Self::IdentityHandle,
            _remote_identity_token: IdentityToken,
            _remote_auth_request_token: Option<AuthRequestMessageToken>,
            _remote_participant_guid: Guid,
        ) -> Result<ValidateRemoteIdentityOut<Self::IdentityHandle>, ValidationResult> {
            if *local_identity_handle == 1 {
                Ok(ValidateRemoteIdentityOut {
                    remote_identity_handle: 100,
                    local_auth_request_token: AuthRequestMessageToken::default(),
                })
            } else {
                Err(ValidationResult::ValidationFailed(
                    SecurityException::default(),
                ))
            }
        }

        fn begin_handshake_request(
            &mut self,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeRequestOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_reply(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeReplyOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn process_handshake(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<HandshakeMessageToken, ValidationResult> {
            unimplemented!()
        }

        fn get_shared_secret(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<Self::SharedSecretHandle, SecurityException> {
            unimplemented!()
        }

        fn get_authenticated_peer_credential_token(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<AuthenticatedPeerCredentialToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_status_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityStatusToken, SecurityException> {
            unimplemented!()
        }

        fn set_participant_security_config(
            &mut self,
            _handle: &Self::IdentityHandle,
            _participant_security_config: &ParticipantSecurityConfig,
        ) -> Result<ParticipantSecurityAlgorithmInfo, SecurityException> {
            unimplemented!()
        }

        fn set_permissions_credential_and_token(
            &mut self,
            _handle: &Self::IdentityHandle,
            _permissions_credential_token: PermissionsCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn set_listener<L>(&mut self, _listener: Option<L>) -> Result<(), SecurityException>
        where
            L: AuthenticationListener<IdentityHandle = Self::IdentityHandle>,
        {
            unimplemented!()
        }

        fn return_identity_token(
            &mut self,
            _token: IdentityToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_status_token(
            &mut self,
            _token: IdentityStatusToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_authenticated_peer_credential_token(
            &mut self,
            _peer_credential_token: AuthenticatedPeerCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_handshake_handle(
            &mut self,
            _handshake_handle: Self::HandshakeHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_handle(
            &mut self,
            _identity_handle: Self::IdentityHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_sharedsecret_handle(
            &mut self,
            _sharedsecret_handle: Self::SharedSecretHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }
    }

    let mut auth = MockAuthentication;
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
    struct MockAuthentication;
    impl Authentication for MockAuthentication {
        type IdentityHandle = u32;
        type HandshakeHandle = u64;
        type SharedSecretHandle = ();

        fn validate_local_identity(
            &mut self,
            _domain_id: DomainId,
            _participant_qos: &DomainParticipantQos,
            _candidate_participant_guid: Guid,
        ) -> Result<ValidateLocalIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn validate_remote_identity(
            &mut self,
            _local_identity_handle: &Self::IdentityHandle,
            _remote_identity_token: IdentityToken,
            _remote_auth_request_token: Option<AuthRequestMessageToken>,
            _remote_participant_guid: Guid,
        ) -> Result<ValidateRemoteIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_request(
            &mut self,
            initiator_identity_handle: &Self::IdentityHandle,
            replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeRequestOut<Self::HandshakeHandle>, ValidationResult> {
            if *initiator_identity_handle == 1 && *replier_identity_handle == 2 {
                Ok(BeginHandshakeRequestOut {
                    handshake_handle: 777,
                    handshake_message_token: HandshakeMessageToken::default(),
                })
            } else {
                Err(ValidationResult::ValidationFailed(
                    SecurityException::default(),
                ))
            }
        }

        fn begin_handshake_reply(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeReplyOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn process_handshake(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<HandshakeMessageToken, ValidationResult> {
            unimplemented!()
        }

        fn get_shared_secret(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<Self::SharedSecretHandle, SecurityException> {
            unimplemented!()
        }

        fn get_authenticated_peer_credential_token(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<AuthenticatedPeerCredentialToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_status_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityStatusToken, SecurityException> {
            unimplemented!()
        }

        fn set_participant_security_config(
            &mut self,
            _handle: &Self::IdentityHandle,
            _participant_security_config: &ParticipantSecurityConfig,
        ) -> Result<ParticipantSecurityAlgorithmInfo, SecurityException> {
            unimplemented!()
        }

        fn set_permissions_credential_and_token(
            &mut self,
            _handle: &Self::IdentityHandle,
            _permissions_credential_token: PermissionsCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn set_listener<L>(&mut self, _listener: Option<L>) -> Result<(), SecurityException>
        where
            L: AuthenticationListener<IdentityHandle = Self::IdentityHandle>,
        {
            unimplemented!()
        }

        fn return_identity_token(
            &mut self,
            _token: IdentityToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_status_token(
            &mut self,
            _token: IdentityStatusToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_authenticated_peer_credential_token(
            &mut self,
            _peer_credential_token: AuthenticatedPeerCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_handshake_handle(
            &mut self,
            _handshake_handle: Self::HandshakeHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_handle(
            &mut self,
            _identity_handle: Self::IdentityHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_sharedsecret_handle(
            &mut self,
            _sharedsecret_handle: Self::SharedSecretHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }
    }

    let mut auth = MockAuthentication;
    let res = auth.begin_handshake_request(&1, &2, &[]);
    assert!(res.is_ok());
    assert_eq!(res.unwrap().handshake_handle, 777);

    let res_err = auth.begin_handshake_request(&1, &99, &[]);
    assert!(res_err.is_err());
}

#[test]
fn begin_handshake_reply_returns_expected_result() {
    struct MockAuthentication;
    impl Authentication for MockAuthentication {
        type IdentityHandle = u32;
        type HandshakeHandle = u64;
        type SharedSecretHandle = ();

        fn validate_local_identity(
            &mut self,
            _domain_id: DomainId,
            _participant_qos: &DomainParticipantQos,
            _candidate_participant_guid: Guid,
        ) -> Result<ValidateLocalIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn validate_remote_identity(
            &mut self,
            _local_identity_handle: &Self::IdentityHandle,
            _remote_identity_token: IdentityToken,
            _remote_auth_request_token: Option<AuthRequestMessageToken>,
            _remote_participant_guid: Guid,
        ) -> Result<ValidateRemoteIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_request(
            &mut self,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeRequestOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_reply(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            initiator_identity_handle: &Self::IdentityHandle,
            replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeReplyOut<Self::HandshakeHandle>, ValidationResult> {
            if *initiator_identity_handle == 10 && *replier_identity_handle == 20 {
                Ok(BeginHandshakeReplyOut {
                    handshake_handle: 888,
                    handshake_message_out: HandshakeMessageToken::default(),
                })
            } else {
                Err(ValidationResult::ValidationFailed(
                    SecurityException::default(),
                ))
            }
        }

        fn process_handshake(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<HandshakeMessageToken, ValidationResult> {
            unimplemented!()
        }

        fn get_shared_secret(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<Self::SharedSecretHandle, SecurityException> {
            unimplemented!()
        }

        fn get_authenticated_peer_credential_token(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<AuthenticatedPeerCredentialToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_status_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityStatusToken, SecurityException> {
            unimplemented!()
        }

        fn set_participant_security_config(
            &mut self,
            _handle: &Self::IdentityHandle,
            _participant_security_config: &ParticipantSecurityConfig,
        ) -> Result<ParticipantSecurityAlgorithmInfo, SecurityException> {
            unimplemented!()
        }

        fn set_permissions_credential_and_token(
            &mut self,
            _handle: &Self::IdentityHandle,
            _permissions_credential_token: PermissionsCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn set_listener<L>(&mut self, _listener: Option<L>) -> Result<(), SecurityException>
        where
            L: AuthenticationListener<IdentityHandle = Self::IdentityHandle>,
        {
            unimplemented!()
        }

        fn return_identity_token(
            &mut self,
            _token: IdentityToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_status_token(
            &mut self,
            _token: IdentityStatusToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_authenticated_peer_credential_token(
            &mut self,
            _peer_credential_token: AuthenticatedPeerCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_handshake_handle(
            &mut self,
            _handshake_handle: Self::HandshakeHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_handle(
            &mut self,
            _identity_handle: Self::IdentityHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_sharedsecret_handle(
            &mut self,
            _sharedsecret_handle: Self::SharedSecretHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }
    }

    let mut auth = MockAuthentication;
    let res = auth.begin_handshake_reply(HandshakeMessageToken::default(), &10, &20, &[]);
    assert!(res.is_ok());
    assert_eq!(res.unwrap().handshake_handle, 888);

    let res_err = auth.begin_handshake_reply(HandshakeMessageToken::default(), &10, &99, &[]);
    assert!(res_err.is_err());
}

#[test]
fn process_handshake_returns_expected_result() {
    struct MockAuthentication;
    impl Authentication for MockAuthentication {
        type IdentityHandle = u32;
        type HandshakeHandle = u64;
        type SharedSecretHandle = u128;

        fn validate_local_identity(
            &mut self,
            _domain_id: DomainId,
            _participant_qos: &DomainParticipantQos,
            _candidate_participant_guid: Guid,
        ) -> Result<ValidateLocalIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn validate_remote_identity(
            &mut self,
            _local_identity_handle: &Self::IdentityHandle,
            _remote_identity_token: IdentityToken,
            _remote_auth_request_token: Option<AuthRequestMessageToken>,
            _remote_participant_guid: Guid,
        ) -> Result<ValidateRemoteIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_request(
            &mut self,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeRequestOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_reply(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeReplyOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn process_handshake(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            handshake_handle: &Self::HandshakeHandle,
        ) -> Result<HandshakeMessageToken, ValidationResult> {
            if *handshake_handle == 777 {
                Ok(HandshakeMessageToken::default())
            } else {
                Err(ValidationResult::ValidationFailed(
                    SecurityException::default(),
                ))
            }
        }

        fn get_shared_secret(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<Self::SharedSecretHandle, SecurityException> {
            unimplemented!()
        }

        fn get_authenticated_peer_credential_token(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<AuthenticatedPeerCredentialToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_status_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityStatusToken, SecurityException> {
            unimplemented!()
        }

        fn set_participant_security_config(
            &mut self,
            _handle: &Self::IdentityHandle,
            _participant_security_config: &ParticipantSecurityConfig,
        ) -> Result<ParticipantSecurityAlgorithmInfo, SecurityException> {
            unimplemented!()
        }

        fn set_permissions_credential_and_token(
            &mut self,
            _handle: &Self::IdentityHandle,
            _permissions_credential_token: PermissionsCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn set_listener<L>(&mut self, _listener: Option<L>) -> Result<(), SecurityException>
        where
            L: AuthenticationListener<IdentityHandle = Self::IdentityHandle>,
        {
            unimplemented!()
        }

        fn return_identity_token(
            &mut self,
            _token: IdentityToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_status_token(
            &mut self,
            _token: IdentityStatusToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_authenticated_peer_credential_token(
            &mut self,
            _peer_credential_token: AuthenticatedPeerCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_handshake_handle(
            &mut self,
            _handshake_handle: Self::HandshakeHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_handle(
            &mut self,
            _identity_handle: Self::IdentityHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_sharedsecret_handle(
            &mut self,
            _sharedsecret_handle: Self::SharedSecretHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }
    }

    let mut auth = MockAuthentication;
    let res = auth.process_handshake(HandshakeMessageToken::default(), &777);
    assert!(res.is_ok());

    let res_err = auth.process_handshake(HandshakeMessageToken::default(), &999);
    assert!(res_err.is_err());
}

#[test]
fn get_shared_secret_returns_expected_result() {
    struct MockAuthentication;
    impl Authentication for MockAuthentication {
        type IdentityHandle = u32;
        type HandshakeHandle = u64;
        type SharedSecretHandle = u128;

        fn validate_local_identity(
            &mut self,
            _domain_id: DomainId,
            _participant_qos: &DomainParticipantQos,
            _candidate_participant_guid: Guid,
        ) -> Result<ValidateLocalIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn validate_remote_identity(
            &mut self,
            _local_identity_handle: &Self::IdentityHandle,
            _remote_identity_token: IdentityToken,
            _remote_auth_request_token: Option<AuthRequestMessageToken>,
            _remote_participant_guid: Guid,
        ) -> Result<ValidateRemoteIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_request(
            &mut self,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeRequestOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_reply(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeReplyOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn process_handshake(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<HandshakeMessageToken, ValidationResult> {
            unimplemented!()
        }

        fn get_shared_secret(
            &mut self,
            handshake_handle: &Self::HandshakeHandle,
        ) -> Result<Self::SharedSecretHandle, SecurityException> {
            if *handshake_handle == 12345 {
                Ok(99999)
            } else {
                Err(SecurityException {
                    message: "Invalid handshake handle".into(),
                    code: 1,
                    minor_code: 0,
                })
            }
        }

        fn get_authenticated_peer_credential_token(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<AuthenticatedPeerCredentialToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_status_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityStatusToken, SecurityException> {
            unimplemented!()
        }

        fn set_participant_security_config(
            &mut self,
            _handle: &Self::IdentityHandle,
            _participant_security_config: &ParticipantSecurityConfig,
        ) -> Result<ParticipantSecurityAlgorithmInfo, SecurityException> {
            unimplemented!()
        }

        fn set_permissions_credential_and_token(
            &mut self,
            _handle: &Self::IdentityHandle,
            _permissions_credential_token: PermissionsCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn set_listener<L>(&mut self, _listener: Option<L>) -> Result<(), SecurityException>
        where
            L: AuthenticationListener<IdentityHandle = Self::IdentityHandle>,
        {
            unimplemented!()
        }

        fn return_identity_token(
            &mut self,
            _token: IdentityToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_status_token(
            &mut self,
            _token: IdentityStatusToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_authenticated_peer_credential_token(
            &mut self,
            _peer_credential_token: AuthenticatedPeerCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_handshake_handle(
            &mut self,
            _handshake_handle: Self::HandshakeHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_handle(
            &mut self,
            _identity_handle: Self::IdentityHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_sharedsecret_handle(
            &mut self,
            _sharedsecret_handle: Self::SharedSecretHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }
    }

    let mut auth = MockAuthentication;
    let res = auth.get_shared_secret(&12345);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 99999);

    let res_err = auth.get_shared_secret(&111);
    assert!(res_err.is_err());
}

#[test]
fn get_authenticated_peer_credential_token_returns_expected_result() {
    struct MockAuthentication;
    impl Authentication for MockAuthentication {
        type IdentityHandle = u32;
        type HandshakeHandle = u64;
        type SharedSecretHandle = u128;

        fn validate_local_identity(
            &mut self,
            _domain_id: DomainId,
            _participant_qos: &DomainParticipantQos,
            _candidate_participant_guid: Guid,
        ) -> Result<ValidateLocalIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn validate_remote_identity(
            &mut self,
            _local_identity_handle: &Self::IdentityHandle,
            _remote_identity_token: IdentityToken,
            _remote_auth_request_token: Option<AuthRequestMessageToken>,
            _remote_participant_guid: Guid,
        ) -> Result<ValidateRemoteIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_request(
            &mut self,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeRequestOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_reply(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeReplyOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn process_handshake(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<HandshakeMessageToken, ValidationResult> {
            unimplemented!()
        }

        fn get_shared_secret(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<Self::SharedSecretHandle, SecurityException> {
            unimplemented!()
        }

        fn get_authenticated_peer_credential_token(
            &mut self,
            handshake_handle: &Self::HandshakeHandle,
        ) -> Result<AuthenticatedPeerCredentialToken, SecurityException> {
            if *handshake_handle == 5555 {
                Ok(AuthenticatedPeerCredentialToken::default())
            } else {
                Err(SecurityException {
                    message: "Invalid handshake handle".into(),
                    code: 1,
                    minor_code: 0,
                })
            }
        }

        fn get_identity_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_status_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityStatusToken, SecurityException> {
            unimplemented!()
        }

        fn set_participant_security_config(
            &mut self,
            _handle: &Self::IdentityHandle,
            _participant_security_config: &ParticipantSecurityConfig,
        ) -> Result<ParticipantSecurityAlgorithmInfo, SecurityException> {
            unimplemented!()
        }

        fn set_permissions_credential_and_token(
            &mut self,
            _handle: &Self::IdentityHandle,
            _permissions_credential_token: PermissionsCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn set_listener<L>(&mut self, _listener: Option<L>) -> Result<(), SecurityException>
        where
            L: AuthenticationListener<IdentityHandle = Self::IdentityHandle>,
        {
            unimplemented!()
        }

        fn return_identity_token(
            &mut self,
            _token: IdentityToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_status_token(
            &mut self,
            _token: IdentityStatusToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_authenticated_peer_credential_token(
            &mut self,
            _peer_credential_token: AuthenticatedPeerCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_handshake_handle(
            &mut self,
            _handshake_handle: Self::HandshakeHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_handle(
            &mut self,
            _identity_handle: Self::IdentityHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_sharedsecret_handle(
            &mut self,
            _sharedsecret_handle: Self::SharedSecretHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }
    }

    let mut auth = MockAuthentication;
    let res = auth.get_authenticated_peer_credential_token(&5555);
    assert!(res.is_ok());

    let res_err = auth.get_authenticated_peer_credential_token(&111);
    assert!(res_err.is_err());
}

#[test]
fn get_identity_status_token_returns_expected_token() {
    struct MockAuthentication;
    impl Authentication for MockAuthentication {
        type IdentityHandle = u32;
        type HandshakeHandle = u64;
        type SharedSecretHandle = u128;

        fn validate_local_identity(
            &mut self,
            _domain_id: DomainId,
            _participant_qos: &DomainParticipantQos,
            _candidate_participant_guid: Guid,
        ) -> Result<ValidateLocalIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn validate_remote_identity(
            &mut self,
            _local_identity_handle: &Self::IdentityHandle,
            _remote_identity_token: IdentityToken,
            _remote_auth_request_token: Option<AuthRequestMessageToken>,
            _remote_participant_guid: Guid,
        ) -> Result<ValidateRemoteIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_request(
            &mut self,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeRequestOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_reply(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeReplyOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn process_handshake(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<HandshakeMessageToken, ValidationResult> {
            unimplemented!()
        }

        fn get_shared_secret(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<Self::SharedSecretHandle, SecurityException> {
            unimplemented!()
        }

        fn get_authenticated_peer_credential_token(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<AuthenticatedPeerCredentialToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_status_token(
            &mut self,
            handle: &Self::IdentityHandle,
        ) -> Result<IdentityStatusToken, SecurityException> {
            if *handle == 100 {
                Ok(IdentityStatusToken::default())
            } else {
                Err(SecurityException {
                    message: "Invalid handle".into(),
                    code: 1,
                    minor_code: 0,
                })
            }
        }

        fn set_participant_security_config(
            &mut self,
            _handle: &Self::IdentityHandle,
            _participant_security_config: &ParticipantSecurityConfig,
        ) -> Result<ParticipantSecurityAlgorithmInfo, SecurityException> {
            unimplemented!()
        }

        fn set_permissions_credential_and_token(
            &mut self,
            _handle: &Self::IdentityHandle,
            _permissions_credential_token: PermissionsCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn set_listener<L>(&mut self, _listener: Option<L>) -> Result<(), SecurityException>
        where
            L: AuthenticationListener<IdentityHandle = Self::IdentityHandle>,
        {
            unimplemented!()
        }

        fn return_identity_token(
            &mut self,
            _token: IdentityToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_status_token(
            &mut self,
            _token: IdentityStatusToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_authenticated_peer_credential_token(
            &mut self,
            _peer_credential_token: AuthenticatedPeerCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_handshake_handle(
            &mut self,
            _handshake_handle: Self::HandshakeHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_handle(
            &mut self,
            _identity_handle: Self::IdentityHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_sharedsecret_handle(
            &mut self,
            _sharedsecret_handle: Self::SharedSecretHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }
    }

    let mut auth = MockAuthentication;
    let res = auth.get_identity_status_token(&100);
    assert!(res.is_ok());

    let res_err = auth.get_identity_status_token(&999);
    assert!(res_err.is_err());
}

#[test]
fn set_participant_security_config_returns_expected_result() {
    struct MockAuthentication;
    impl Authentication for MockAuthentication {
        type IdentityHandle = u32;
        type HandshakeHandle = u64;
        type SharedSecretHandle = u128;

        fn validate_local_identity(
            &mut self,
            _domain_id: DomainId,
            _participant_qos: &DomainParticipantQos,
            _candidate_participant_guid: Guid,
        ) -> Result<ValidateLocalIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn validate_remote_identity(
            &mut self,
            _local_identity_handle: &Self::IdentityHandle,
            _remote_identity_token: IdentityToken,
            _remote_auth_request_token: Option<AuthRequestMessageToken>,
            _remote_participant_guid: Guid,
        ) -> Result<ValidateRemoteIdentityOut<Self::IdentityHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_request(
            &mut self,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeRequestOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn begin_handshake_reply(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _initiator_identity_handle: &Self::IdentityHandle,
            _replier_identity_handle: &Self::IdentityHandle,
            _serialized_local_participant_data: &[u8],
        ) -> Result<BeginHandshakeReplyOut<Self::HandshakeHandle>, ValidationResult> {
            unimplemented!()
        }

        fn process_handshake(
            &mut self,
            _handshake_message_in: HandshakeMessageToken,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<HandshakeMessageToken, ValidationResult> {
            unimplemented!()
        }

        fn get_shared_secret(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<Self::SharedSecretHandle, SecurityException> {
            unimplemented!()
        }

        fn get_authenticated_peer_credential_token(
            &mut self,
            _handshake_handle: &Self::HandshakeHandle,
        ) -> Result<AuthenticatedPeerCredentialToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityToken, SecurityException> {
            unimplemented!()
        }

        fn get_identity_status_token(
            &mut self,
            _handle: &Self::IdentityHandle,
        ) -> Result<IdentityStatusToken, SecurityException> {
            unimplemented!()
        }

        fn set_participant_security_config(
            &mut self,
            handle: &Self::IdentityHandle,
            participant_security_config: &ParticipantSecurityConfig,
        ) -> Result<ParticipantSecurityAlgorithmInfo, SecurityException> {
            if *handle == 10 {
                Ok(participant_security_config.algorithm_info)
            } else {
                Err(SecurityException {
                    message: "Invalid handle".into(),
                    code: 1,
                    minor_code: 0,
                })
            }
        }

        fn set_permissions_credential_and_token(
            &mut self,
            _handle: &Self::IdentityHandle,
            _permissions_credential_token: PermissionsCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn set_listener<L>(&mut self, _listener: Option<L>) -> Result<(), SecurityException>
        where
            L: AuthenticationListener<IdentityHandle = Self::IdentityHandle>,
        {
            unimplemented!()
        }

        fn return_identity_token(
            &mut self,
            _token: IdentityToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_status_token(
            &mut self,
            _token: IdentityStatusToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_authenticated_peer_credential_token(
            &mut self,
            _peer_credential_token: AuthenticatedPeerCredentialToken,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_handshake_handle(
            &mut self,
            _handshake_handle: Self::HandshakeHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_identity_handle(
            &mut self,
            _identity_handle: Self::IdentityHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }

        fn return_sharedsecret_handle(
            &mut self,
            _sharedsecret_handle: Self::SharedSecretHandle,
        ) -> Result<(), SecurityException> {
            unimplemented!()
        }
    }

    let mut auth = MockAuthentication;
    let config = ParticipantSecurityConfig::default();
    let res = auth.set_participant_security_config(&10, &config);
    assert!(res.is_ok());

    let res_err = auth.set_participant_security_config(&999, &config);
    assert!(res_err.is_err());
}
