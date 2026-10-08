use dust_dds::{
    builtin_topics::{
        ParticipantBuiltinTopicData, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData,
        TopicBuiltinTopicData,
    },
    infrastructure::{
        domain::DomainId,
        instance::InstanceHandle,
        qos::{DataReaderQos, DataWriterQos, DomainParticipantQos, TopicQos},
        qos_policy::{DataTagQosPolicy, PartitionQosPolicy},
    },
    security::{
        plugins::{
            access_control::{AccessControl, CheckRemoteDataReaderOut},
            access_control_listener::AccessControlListener,
            authentication::{
                Authentication, BeginHandshakeReplyOut, BeginHandshakeRequestOut,
                ValidateLocalIdentityOut, ValidateRemoteIdentityOut, ValidationResult,
            },
            authentication_listener::AuthenticationListener,
            types::SecurityException,
        },
        types::{
            AuthRequestMessageToken, AuthenticatedPeerCredentialToken, EndpointSecurityConfig,
            HandshakeMessageToken, IdentityStatusToken, IdentityToken,
            ParticipantSecurityAlgorithmInfo, ParticipantSecurityConfig,
            PermissionsCredentialToken, PermissionsToken, TopicSecurityConfig,
        },
    },
    transport::types::Guid,
};

#[derive(Default)]
pub struct StubAuthentication {
    pub validate_local_identity_fn: Option<
        Box<
            dyn FnMut(
                    DomainId,
                    &DomainParticipantQos,
                    Guid,
                ) -> Result<ValidateLocalIdentityOut<u32>, ValidationResult>
                + Send,
        >,
    >,
    pub validate_remote_identity_fn: Option<
        Box<
            dyn FnMut(
                    &u32,
                    IdentityToken,
                    Option<AuthRequestMessageToken>,
                    Guid,
                ) -> Result<ValidateRemoteIdentityOut<u32>, ValidationResult>
                + Send,
        >,
    >,
    pub begin_handshake_request_fn: Option<
        Box<
            dyn FnMut(&u32, &u32, &[u8]) -> Result<BeginHandshakeRequestOut<u64>, ValidationResult>
                + Send,
        >,
    >,
    pub begin_handshake_reply_fn: Option<
        Box<
            dyn FnMut(
                    HandshakeMessageToken,
                    &u32,
                    &u32,
                    &[u8],
                ) -> Result<BeginHandshakeReplyOut<u64>, ValidationResult>
                + Send,
        >,
    >,
    pub process_handshake_fn: Option<
        Box<
            dyn FnMut(
                    HandshakeMessageToken,
                    &u64,
                ) -> Result<HandshakeMessageToken, ValidationResult>
                + Send,
        >,
    >,
    pub get_shared_secret_fn:
        Option<Box<dyn FnMut(&u64) -> Result<u128, SecurityException> + Send>>,
    pub get_authenticated_peer_credential_token_fn: Option<
        Box<dyn FnMut(&u64) -> Result<AuthenticatedPeerCredentialToken, SecurityException> + Send>,
    >,
    pub get_identity_token_fn:
        Option<Box<dyn FnMut(&u32) -> Result<IdentityToken, SecurityException> + Send>>,
    pub get_identity_status_token_fn: Option<
        Box<dyn FnMut(&u32) -> Result<Option<IdentityStatusToken>, SecurityException> + Send>,
    >,
    pub set_participant_security_config_fn: Option<
        Box<
            dyn FnMut(
                    &u32,
                    &ParticipantSecurityConfig,
                )
                    -> Result<ParticipantSecurityAlgorithmInfo, SecurityException>
                + Send,
        >,
    >,
}

impl Authentication for StubAuthentication {
    type IdentityHandle = u32;
    type HandshakeHandle = u64;
    type SharedSecretHandle = u128;

    fn validate_local_identity(
        &mut self,
        domain_id: DomainId,
        participant_qos: &DomainParticipantQos,
        candidate_participant_guid: Guid,
    ) -> Result<ValidateLocalIdentityOut<Self::IdentityHandle>, ValidationResult> {
        if let Some(ref mut f) = self.validate_local_identity_fn {
            f(domain_id, participant_qos, candidate_participant_guid)
        } else {
            Ok(ValidateLocalIdentityOut {
                local_identity_handle: 1,
                adjusted_participant_guid: candidate_participant_guid,
            })
        }
    }

    fn validate_remote_identity(
        &mut self,
        local_identity_handle: &Self::IdentityHandle,
        remote_identity_token: IdentityToken,
        remote_auth_request_token: Option<AuthRequestMessageToken>,
        remote_participant_guid: Guid,
    ) -> Result<ValidateRemoteIdentityOut<Self::IdentityHandle>, ValidationResult> {
        if let Some(ref mut f) = self.validate_remote_identity_fn {
            f(
                local_identity_handle,
                remote_identity_token,
                remote_auth_request_token,
                remote_participant_guid,
            )
        } else {
            Ok(ValidateRemoteIdentityOut {
                remote_identity_handle: 1,
                local_auth_request_token: AuthRequestMessageToken::default(),
            })
        }
    }

    fn begin_handshake_request(
        &mut self,
        initiator_identity_handle: &Self::IdentityHandle,
        replier_identity_handle: &Self::IdentityHandle,
        serialized_local_participant_data: &[u8],
    ) -> Result<BeginHandshakeRequestOut<Self::HandshakeHandle>, ValidationResult> {
        if let Some(ref mut f) = self.begin_handshake_request_fn {
            f(
                initiator_identity_handle,
                replier_identity_handle,
                serialized_local_participant_data,
            )
        } else {
            Ok(BeginHandshakeRequestOut {
                handshake_handle: 1,
                handshake_message_token: HandshakeMessageToken::default(),
            })
        }
    }

    fn begin_handshake_reply(
        &mut self,
        handshake_message_in: HandshakeMessageToken,
        initiator_identity_handle: &Self::IdentityHandle,
        replier_identity_handle: &Self::IdentityHandle,
        serialized_local_participant_data: &[u8],
    ) -> Result<BeginHandshakeReplyOut<Self::HandshakeHandle>, ValidationResult> {
        if let Some(ref mut f) = self.begin_handshake_reply_fn {
            f(
                handshake_message_in,
                initiator_identity_handle,
                replier_identity_handle,
                serialized_local_participant_data,
            )
        } else {
            Ok(BeginHandshakeReplyOut {
                handshake_handle: 1,
                handshake_message_out: HandshakeMessageToken::default(),
            })
        }
    }

    fn process_handshake(
        &mut self,
        handshake_message_in: HandshakeMessageToken,
        handshake_handle: &Self::HandshakeHandle,
    ) -> Result<HandshakeMessageToken, ValidationResult> {
        if let Some(ref mut f) = self.process_handshake_fn {
            f(handshake_message_in, handshake_handle)
        } else {
            Ok(HandshakeMessageToken::default())
        }
    }

    fn get_shared_secret(
        &mut self,
        handshake_handle: &Self::HandshakeHandle,
    ) -> Result<Self::SharedSecretHandle, SecurityException> {
        if let Some(ref mut f) = self.get_shared_secret_fn {
            f(handshake_handle)
        } else {
            Ok(1)
        }
    }

    fn get_authenticated_peer_credential_token(
        &mut self,
        handshake_handle: &Self::HandshakeHandle,
    ) -> Result<AuthenticatedPeerCredentialToken, SecurityException> {
        if let Some(ref mut f) = self.get_authenticated_peer_credential_token_fn {
            f(handshake_handle)
        } else {
            Ok(AuthenticatedPeerCredentialToken::default())
        }
    }

    fn get_identity_token(
        &mut self,
        handle: &Self::IdentityHandle,
    ) -> Result<IdentityToken, SecurityException> {
        if let Some(ref mut f) = self.get_identity_token_fn {
            f(handle)
        } else {
            Ok(IdentityToken::default())
        }
    }

    fn get_identity_status_token(
        &mut self,
        handle: &Self::IdentityHandle,
    ) -> Result<Option<IdentityStatusToken>, SecurityException> {
        if let Some(ref mut f) = self.get_identity_status_token_fn {
            f(handle)
        } else {
            Ok(Some(IdentityStatusToken::default()))
        }
    }

    fn set_participant_security_config(
        &mut self,
        handle: &Self::IdentityHandle,
        participant_security_config: &ParticipantSecurityConfig,
    ) -> Result<ParticipantSecurityAlgorithmInfo, SecurityException> {
        if let Some(ref mut f) = self.set_participant_security_config_fn {
            f(handle, participant_security_config)
        } else {
            Ok(ParticipantSecurityAlgorithmInfo::default())
        }
    }

    fn set_permissions_credential_and_token(
        &mut self,
        _handle: &Self::IdentityHandle,
        _permissions_credential_token: PermissionsCredentialToken,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn set_listener<L>(&mut self, _listener: Option<L>) -> Result<(), SecurityException>
    where
        L: AuthenticationListener<IdentityHandle = Self::IdentityHandle>,
    {
        Ok(())
    }

    fn return_identity_token(&mut self, _token: IdentityToken) -> Result<(), SecurityException> {
        Ok(())
    }

    fn return_identity_status_token(
        &mut self,
        _token: IdentityStatusToken,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn return_authenticated_peer_credential_token(
        &mut self,
        _peer_credential_token: AuthenticatedPeerCredentialToken,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn return_handshake_handle(
        &mut self,
        _handshake_handle: Self::HandshakeHandle,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn return_identity_handle(
        &mut self,
        _identity_handle: Self::IdentityHandle,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn return_sharedsecret_handle(
        &mut self,
        _sharedsecret_handle: Self::SharedSecretHandle,
    ) -> Result<(), SecurityException> {
        Ok(())
    }
}

#[derive(Default)]
pub struct StubAccessControl {
    pub validate_local_permissions_fn: Option<
        Box<dyn FnMut(DomainId, &DomainParticipantQos) -> Result<u32, SecurityException> + Send>,
    >,
    pub check_create_participant_fn: Option<
        Box<
            dyn FnMut(&u32, DomainId, &DomainParticipantQos) -> Result<(), SecurityException>
                + Send,
        >,
    >,
}

impl AccessControl for StubAccessControl {
    type PermissionsHandle = u32;

    fn validate_local_permissions<A>(
        &mut self,
        _auth_plugin: &mut A,
        _identity: &A::IdentityHandle,
        domain_id: DomainId,
        participant_qos: &DomainParticipantQos,
    ) -> Result<Self::PermissionsHandle, SecurityException>
    where
        A: Authentication,
    {
        if let Some(ref mut f) = self.validate_local_permissions_fn {
            f(domain_id, participant_qos)
        } else {
            Ok(1)
        }
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
        Ok(1)
    }

    fn check_create_participant(
        &mut self,
        permissions_handle: &Self::PermissionsHandle,
        domain_id: DomainId,
        qos: &DomainParticipantQos,
    ) -> Result<(), SecurityException> {
        if let Some(ref mut f) = self.check_create_participant_fn {
            f(permissions_handle, domain_id, qos)
        } else {
            Ok(())
        }
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
        Ok(())
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
        Ok(())
    }

    fn check_create_topic(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _domain_id: DomainId,
        _topic_name: &str,
        _qos: &TopicQos,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn check_local_datawriter_register_instance(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _writer: &PublicationBuiltinTopicData,
        _key: &InstanceHandle,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn check_local_datawriter_dispose_instance(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _writer: &PublicationBuiltinTopicData,
        _key: &InstanceHandle,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn check_remote_participant(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _domain_id: DomainId,
        _participant_data: &ParticipantBuiltinTopicData,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn check_remote_datawriter(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _domain_id: DomainId,
        _publication_data: &PublicationBuiltinTopicData,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn check_remote_datareader(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _domain_id: DomainId,
        _subscription_data: &SubscriptionBuiltinTopicData,
    ) -> Result<CheckRemoteDataReaderOut, SecurityException> {
        Ok(CheckRemoteDataReaderOut::default())
    }

    fn check_remote_topic(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _domain_id: DomainId,
        _topic_data: &TopicBuiltinTopicData,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn check_local_datawriter_match(
        &mut self,
        _writer_permissions_handle: &Self::PermissionsHandle,
        _reader_permissions_handle: &Self::PermissionsHandle,
        _publication_data: &PublicationBuiltinTopicData,
        _subscription_data: &SubscriptionBuiltinTopicData,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn check_local_datareader_match(
        &mut self,
        _reader_permissions_handle: &Self::PermissionsHandle,
        _writer_permissions_handle: &Self::PermissionsHandle,
        _subscription_data: &SubscriptionBuiltinTopicData,
        _publication_data: &PublicationBuiltinTopicData,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn check_remote_datawriter_register_instance(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _reader: &SubscriptionBuiltinTopicData,
        _publication_handle: &InstanceHandle,
        _key: &InstanceHandle,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn check_remote_datawriter_dispose_instance(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _reader: &SubscriptionBuiltinTopicData,
        _publication_handle: &InstanceHandle,
        _key: &InstanceHandle,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn get_permissions_token(
        &mut self,
        _handle: &Self::PermissionsHandle,
    ) -> Result<PermissionsToken, SecurityException> {
        Ok(PermissionsToken::default())
    }

    fn get_permissions_credential_token(
        &mut self,
        _handle: &Self::PermissionsHandle,
    ) -> Result<PermissionsCredentialToken, SecurityException> {
        Ok(PermissionsCredentialToken::default())
    }

    fn set_listener<L>(&mut self, _listener: Option<L>) -> Result<(), SecurityException>
    where
        L: AccessControlListener<PermissionsHandle = Self::PermissionsHandle>,
    {
        Ok(())
    }

    fn return_permissions_token(
        &mut self,
        _token: PermissionsToken,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn return_permissions_credential_token(
        &mut self,
        _permissions_credential_token: PermissionsCredentialToken,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn get_participant_security_config(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
    ) -> Result<ParticipantSecurityConfig, SecurityException> {
        Ok(ParticipantSecurityConfig::default())
    }

    fn get_topic_security_config(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _topic_name: &str,
    ) -> Result<TopicSecurityConfig, SecurityException> {
        Ok(TopicSecurityConfig::default())
    }

    fn get_datawriter_security_config(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _topic_name: &str,
        _partition: &PartitionQosPolicy,
        _data_tag: &DataTagQosPolicy,
    ) -> Result<EndpointSecurityConfig, SecurityException> {
        Ok(EndpointSecurityConfig::default())
    }

    fn get_datareader_security_config(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _topic_name: &str,
        _partition: &PartitionQosPolicy,
        _data_tag: &DataTagQosPolicy,
    ) -> Result<EndpointSecurityConfig, SecurityException> {
        Ok(EndpointSecurityConfig::default())
    }

    fn return_participant_security_config(
        &mut self,
        _attributes: ParticipantSecurityConfig,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn return_topic_security_config(
        &mut self,
        _attributes: TopicSecurityConfig,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn return_datawriter_security_config(
        &mut self,
        _attributes: EndpointSecurityConfig,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn return_datareader_security_config(
        &mut self,
        _attributes: EndpointSecurityConfig,
    ) -> Result<(), SecurityException> {
        Ok(())
    }
}
