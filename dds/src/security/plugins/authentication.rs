use super::types::SecurityException;
use crate::{
    infrastructure::{domain::DomainId, qos::DomainParticipantQos},
    security::types::{
        AuthRequestMessageToken, AuthenticatedPeerCredentialToken, HandshakeMessageToken,
        IdentityStatusToken, IdentityToken, ParticipantSecurityAlgorithmInfo,
        ParticipantSecurityConfig,
    },
    transport::types::Guid,
};

/// Outcome of validation operations as defined in Table 30 of Section 9.3.1 of the DDS Security specification (excluding `VALIDATION_OK` which maps to [`Ok`]).
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValidationResult {
    /// Indicates the validation has failed.
    ValidationFailed(SecurityException),
    /// Indicates that validation is still proceeding. The operation shall be retried at a later point in time.
    ValidationPendingRetry,
    /// Indicates that validation of the submitted IdentityToken requires sending a handshake message.
    ValidationPendingHandshakeRequest,
    /// Indicates that validation is still pending. The DDS Implementation shall wait for a message on the BuiltinParticipantMessageReader.
    ValidationPendingHandshakeMessage,
    /// Indicates that validation has succeeded but the DDS Implementation shall send a final message.
    ValidationOkFinalMessage,
}

/// Output parameters for [`Authentication::validate_local_identity`].
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ValidateLocalIdentityOut<I> {
    /// Handle to the validated local identity.
    pub local_identity_handle: I,
    /// Adjusted participant GUID.
    pub adjusted_participant_guid: Guid,
}

/// Output parameters for [`Authentication::validate_remote_identity`].
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ValidateRemoteIdentityOut<I> {
    /// Handle that can be used to locally refer to the remote authenticated participant.
    pub remote_identity_handle: I,
    /// Auth request token to be sent using the `BuiltinParticipantStatelessMessageWriter`.
    pub local_auth_request_token: AuthRequestMessageToken,
}

/// Output parameters for [`Authentication::begin_handshake_request`].
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BeginHandshakeRequestOut<H> {
    /// Handle used to keep the state of the handshake.
    pub handshake_handle: H,
    /// Handshake message token to be sent using the `BuiltinParticipantMessageWriter`.
    pub handshake_message_token: HandshakeMessageToken,
}

/// Output parameters for [`Authentication::begin_handshake_reply`].
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BeginHandshakeReplyOut<H> {
    /// Handle used to keep the state of the handshake.
    pub handshake_handle: H,
    /// Handshake message token containing a message to be sent using the `BuiltinParticipantMessageWriter`.
    pub handshake_message_out: HandshakeMessageToken,
}

/// Authentication plugin interface as defined in Section 9.3.2 of the DDS Security specification.
pub trait Authentication: Send + 'static {
    /// Opaque handle representing internal authentication state as defined in Section 9.3.2.3 of the DDS Security specification.
    type IdentityHandle;

    /// Opaque handle representing internal handshake state as defined in Section 9.3.2.3 of the DDS Security specification.
    type HandshakeHandle;

    /// Opaque handle representing shared secret state as defined in Section 9.3.2.3 of the DDS Security specification.
    type SharedSecretHandle;

    /// Validates the identity of the local `DomainParticipant`.
    ///
    /// The operation returns as an output parameter the [`IdentityHandle`],
    /// which can be used to locally identify the local Participant to the Authentication Plugin.
    ///
    /// In addition to validating the identity, this operation also returns the `DomainParticipant` [`Guid`]
    /// that shall be used by the DDS implementation to uniquely identify the `DomainParticipant` on the network.
    ///
    /// This operation shall be called before the `DomainParticipant` is enabled. It shall be called either
    /// by the implementation of `DomainParticipantFactory::create_domain_participant` or
    /// `DomainParticipant::enable`.
    ///
    /// If an error occurs, this method shall return [`ValidationResult::ValidationFailed`] containing the [`SecurityException`].
    ///
    /// The method shall return either [`Ok`] carrying [`ValidateLocalIdentityOut`] if the validation succeeds, or
    /// [`Err(ValidationResult)`](ValidationResult) if it fails or requires further steps. If [`ValidationResult::ValidationPendingRetry`]
    /// has been returned, the operation shall be called again after a configurable delay to check the status of verification.
    /// This shall continue until the operation returns either [`Ok`] (if the validation succeeds) or [`ValidationResult::ValidationFailed`].
    /// This approach allows non-blocking interactions with services whose verification may require invoking remote services.
    ///
    /// # Arguments
    ///
    /// * `domain_id` - The DDS Domain Id of the `DomainParticipant`.
    /// * `participant_qos` - The [`DomainParticipantQos`] of the `DomainParticipant`.
    /// * `candidate_participant_guid` - The [`Guid`] that the DDS implementation would have
    ///   used to uniquely identify the `DomainParticipant` if the Security plugins were not enabled.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationResult`]:
    /// * [`ValidationResult::ValidationFailed`] - If validation failed. Contains details in a [`SecurityException`].
    /// * [`ValidationResult::ValidationPendingRetry`] - If verification has not completed and the operation should be retried later.
    /// * Other [`ValidationResult`] variants for handshake states.
    fn validate_local_identity(
        &mut self,
        domain_id: DomainId,
        participant_qos: &DomainParticipantQos,
        candidate_participant_guid: Guid,
    ) -> Result<ValidateLocalIdentityOut<Self::IdentityHandle>, ValidationResult>;

    /// Initiates the process of validating the identity of the discovered remote `DomainParticipant`,
    /// represented as an [`IdentityToken`] object.
    ///
    /// The operation returns the [`ValidationResult`] indicating whether the validation succeeded, failed,
    /// or is pending a handshake. If the validation succeeds, a [`ValidateRemoteIdentityOut`] containing the [`IdentityHandle`](Self::IdentityHandle)
    /// object is returned, which can be used to locally identify the remote `DomainParticipant` to the Authentication plugin.
    ///
    /// If the validation can be performed with the information passed and succeeds, the operation shall return
    /// `Ok(`[`ValidateRemoteIdentityOut`]`)`. If it can be performed with the information passed and it fails, it shall return
    /// [`ValidationResult::ValidationFailed`].
    ///
    /// The validation of a remote participant might require the remote participant to perform a handshake. In
    /// this situation, the `validate_remote_identity` operation shall return
    /// [`ValidationResult::ValidationPendingHandshakeRequest`] or
    /// [`ValidationResult::ValidationPendingHandshakeMessage`].
    ///
    /// If the operation returns [`ValidationResult::ValidationPendingHandshakeRequest`], then the DDS
    /// implementation shall call the operation `begin_handshake_request` to continue the validation process.
    ///
    /// If the operation returns [`ValidationResult::ValidationPendingHandshakeMessage`], then the DDS
    /// implementation shall wait until it receives a `ParticipantStatelessMessage` from the remote participant
    /// identified by the `remote_participant_guid` using the contents described in Section 9.3.2.11.5 and
    /// then call the operation `begin_handshake_reply`.
    ///
    /// # Arguments
    ///
    /// * `local_identity_handle` - The handle used to locally identify the local `DomainParticipant`.
    /// * `remote_identity_token` - A token received as part of `ParticipantBuiltinTopicData`, representing
    ///   the identity of the remote `DomainParticipant`.
    /// * `remote_auth_request_token` - The [`AuthRequestMessageToken`] received from the remote `DomainParticipant`
    ///   that caused the authentication to begin. This token shall be [`None`] if the authentication was not
    ///   initiated by the reception of an [`AuthRequestMessageToken`].
    /// * `remote_participant_guid` - [`Guid`] uniquely identifying the remote participant.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationResult`]:
    /// * [`ValidationResult::ValidationFailed`] - If validation failed. Contains details in a [`SecurityException`].
    /// * [`ValidationResult::ValidationPendingHandshakeRequest`] - If validation has not completed. If this is returned,
    ///   the DDS implementation shall call `begin_handshake_request` to continue the validation.
    /// * [`ValidationResult::ValidationPendingHandshakeMessage`] - If validation has not completed. If this is returned,
    ///   the DDS implementation shall wait for a message on the `BuiltinParticipantMessageReader` with the `message_identity`
    ///   containing a `source_guid` that matches the `remote_participant_guid` and a `message_class_id` set to `GMCLASSID_SECURITY_AUTH_HANDSHAKE`.
    /// * [`ValidationResult::ValidationPendingRetry`] - If validation has not completed. If this is returned,
    ///   the operation should be called again at a later point in time to check the validation status.
    fn validate_remote_identity(
        &mut self,
        local_identity_handle: &Self::IdentityHandle,
        remote_identity_token: IdentityToken,
        remote_auth_request_token: Option<AuthRequestMessageToken>,
        remote_participant_guid: Guid,
    ) -> Result<ValidateRemoteIdentityOut<Self::IdentityHandle>, ValidationResult>;

    /// Initiates a handshake process with a remote `DomainParticipant`.
    ///
    /// It shall be called by the DDS middleware solely as a result of having a previous call to
    /// [`validate_remote_identity`](Self::validate_remote_identity) returning
    /// [`ValidationResult::ValidationPendingHandshakeRequest`].
    ///
    /// This operation returns [`BeginHandshakeRequestOut`] containing the [`HandshakeHandle`](Self::HandshakeHandle)
    /// and [`HandshakeMessageToken`] that shall be used to send a handshake to the remote participant identified by the `replier_identity_handle`.
    ///
    /// # Arguments
    ///
    /// * `initiator_identity_handle` - Handle to the local participant that originated the handshake.
    /// * `replier_identity_handle` - Handle to the remote participant whose identity is being validated.
    /// * `serialized_local_participant_data` - CDR Big Endian Serialization for the `ParticipantBuiltInTopicDataSecure` object associated with the local `DomainParticipant`.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationResult`]:
    /// * [`ValidationResult::ValidationFailed`] - If validation failed. Contains details in a [`SecurityException`].
    /// * [`ValidationResult::ValidationPendingHandshakeMessage`] - If validation has not completed. The DDS implementation shall send the returned
    ///   [`HandshakeMessageToken`] using the `BuiltinParticipantMessageWriter` and wait for a reply on the `BuiltinParticipantMessageReader`.
    /// * [`ValidationResult::ValidationOkFinalMessage`] - If validation succeeded and a final handshake message needs to be sent.
    /// * [`ValidationResult::ValidationPendingRetry`] - If validation has not completed and the operation should be retried later.
    fn begin_handshake_request(
        &mut self,
        initiator_identity_handle: &Self::IdentityHandle,
        replier_identity_handle: &Self::IdentityHandle,
        serialized_local_participant_data: &[u8],
    ) -> Result<BeginHandshakeRequestOut<Self::HandshakeHandle>, ValidationResult>;

    /// Responds to the reception of the initial handshake message that originated on a `DomainParticipant` that called `begin_handshake_request`.
    ///
    /// It shall be called by the DDS implementation solely as a result of having a previous call to
    /// [`validate_remote_identity`](Self::validate_remote_identity) returning
    /// [`ValidationResult::ValidationPendingHandshakeMessage`] and having received a message on the `BuiltinParticipantMessageReader`.
    ///
    /// This operation generates a `handshake_message_out` in response to a received `handshake_message_in`.
    ///
    /// # Arguments
    ///
    /// * `handshake_message_in` - A [`HandshakeMessageToken`] containing a message received from the `BuiltinParticipantMessageReader`.
    /// * `initiator_identity_handle` - Handle to the remote participant that originated the handshake.
    /// * `replier_identity_handle` - Handle to the local participant that is initiating the handshake response.
    /// * `serialized_local_participant_data` - CDR Big Endian Serialization for the `ParticipantBuiltInTopicDataSecure` object associated with the local `DomainParticipant`.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationResult`]:
    /// * [`ValidationResult::ValidationFailed`] - If validation failed. Contains details in a [`SecurityException`].
    /// * [`ValidationResult::ValidationPendingHandshakeMessage`] - If validation has not completed. The DDS implementation shall send the `handshake_message_out`
    ///   using the `BuiltinParticipantMessageWriter` and then wait for a reply message on the `BuiltinParticipantMessageReader` from that remote `DomainParticipant`.
    /// * [`ValidationResult::ValidationOkFinalMessage`] - If validation succeeded. The DDS implementation shall send the returned `handshake_message_out`
    ///   using the `BuiltinParticipantMessageWriter`.
    /// * [`ValidationResult::ValidationPendingRetry`] - If validation has not completed and the operation should be called again at a later point in time.
    fn begin_handshake_reply(
        &mut self,
        handshake_message_in: HandshakeMessageToken,
        initiator_identity_handle: &Self::IdentityHandle,
        replier_identity_handle: &Self::IdentityHandle,
        serialized_local_participant_data: &[u8],
    ) -> Result<BeginHandshakeReplyOut<Self::HandshakeHandle>, ValidationResult>;

    /// Continues an in-progress handshake process.
    ///
    /// It shall be called by the DDS middleware solely as a result of having a previous call to
    /// [`begin_handshake_request`](Self::begin_handshake_request) or [`begin_handshake_reply`](Self::begin_handshake_reply)
    /// that returned [`ValidationResult::ValidationPendingHandshakeMessage`] and having received a `ParticipantStatelessMessage`
    /// on the `BuiltinParticipantMessageReader`.
    ///
    /// This operation generates a `handshake_message_out` [`HandshakeMessageToken`] in response to a received `handshake_message_in` [`HandshakeMessageToken`].
    ///
    /// # Arguments
    ///
    /// * `handshake_message_in` - The [`HandshakeMessageToken`] contained in the received message.
    /// * `handshake_handle` - Handle returned by a corresponding previous call to `begin_handshake_request` or `begin_handshake_reply`.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationResult`]:
    /// * [`ValidationResult::ValidationFailed`] - If validation failed. Contains details in a [`SecurityException`].
    /// * [`ValidationResult::ValidationPendingHandshakeMessage`] - If validation has not completed. The DDS implementation shall send the `handshake_message_out`
    ///   and wait for a reply message.
    /// * [`ValidationResult::ValidationOkFinalMessage`] - If validation succeeded and a final message needs to be sent.
    /// * [`ValidationResult::ValidationPendingRetry`] - If validation has not completed and the operation should be retried later.
    fn process_handshake(
        &mut self,
        handshake_message_in: HandshakeMessageToken,
        handshake_handle: &Self::HandshakeHandle,
    ) -> Result<HandshakeMessageToken, ValidationResult>;

    /// Retrieves the [`SharedSecretHandle`](Self::SharedSecretHandle) resulting from a successfully completed handshake.
    ///
    /// This operation shall be called by the DDS middleware on each [`HandshakeHandle`](Self::HandshakeHandle)
    /// after the handshake that uses that handle completes successfully (i.e. after the last handshake operation
    /// called on that handle returns [`Ok`] or [`ValidationResult::ValidationOkFinalMessage`]).
    ///
    /// The retrieved [`SharedSecretHandle`](Self::SharedSecretHandle) shall be used by the DDS middleware in
    /// conjunction with the `CryptoKeyExchange` interface of the Cryptographic Plugin to exchange cryptographic
    /// key material with other `DomainParticipant` entities.
    ///
    /// # Arguments
    ///
    /// * `handshake_handle` - Handle returned by a corresponding previous call to `begin_handshake_request`
    ///   or `begin_handshake_reply`, which has successfully completed the handshake operations.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] if an error occurs.
    fn get_shared_secret(
        &mut self,
        handshake_handle: &Self::HandshakeHandle,
    ) -> Result<Self::SharedSecretHandle, SecurityException>;

    /// Retrieves the [`AuthenticatedPeerCredentialToken`] resulting from a successfully completed
    /// authentication of a discovered `DomainParticipant`.
    ///
    /// This operation shall be called by the DDS middleware on each [`HandshakeHandle`](Self::HandshakeHandle)
    /// after the handshake that uses that handle completes successfully (i.e. after the last handshake operation
    /// called on that handle returns [`Ok`] or [`ValidationResult::ValidationOkFinalMessage`]).
    ///
    /// # Arguments
    ///
    /// * `handshake_handle` - Handle returned by a corresponding previous call to `begin_handshake_request`
    ///   or `begin_handshake_reply`, which has successfully completed the handshake operations.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] if an error occurs.
    fn get_authenticated_peer_credential_token(
        &mut self,
        handshake_handle: &Self::HandshakeHandle,
    ) -> Result<AuthenticatedPeerCredentialToken, SecurityException>;

    /// Retrieves an [`IdentityToken`] used to represent on the network the identity of the
    /// `DomainParticipant` identified by the specified [`IdentityHandle`](Self::IdentityHandle).
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle used to locally identify the `DomainParticipant` for which an
    ///   [`IdentityToken`] is desired. The handle must have been returned by a successful call to
    ///   [`validate_local_identity`](Self::validate_local_identity), otherwise the operation shall return false and fill the [`SecurityException`].
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] if an error occurs. Otherwise, it shall return the [`IdentityToken`].
    fn get_identity_token(
        &mut self,
        handle: &Self::IdentityHandle,
    ) -> Result<IdentityToken, SecurityException>;

    /// Retrieves an [`IdentityStatusToken`] used to represent on the network the authentication state of
    /// the `DomainParticipant` identified by the specified [`IdentityHandle`](Self::IdentityHandle).
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle used to locally identify the `DomainParticipant` for which an
    ///   [`IdentityStatusToken`] is desired. The handle must have been returned by a successful call to
    ///   [`validate_local_identity`](Self::validate_local_identity), otherwise the operation shall return false and fill the [`SecurityException`].
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] if an error occurs. Otherwise, it shall return the [`IdentityStatusToken`].
    fn get_identity_status_token(
        &mut self,
        handle: &Self::IdentityHandle,
    ) -> Result<IdentityStatusToken, SecurityException>;

    /// Configures various aspects of the Authentication algorithm used by the Authentication plugin
    /// and retrieves an updated [`ParticipantSecurityAlgorithmInfo`] that contains the cryptographic
    /// algorithms used and supported by the Authentication plugin.
    ///
    /// The operation shall be called by the middleware after calling [`validate_local_identity`](Self::validate_local_identity)
    /// on the Authentication plugin and calling `get_participant_security_config` on the `AccessControl` plugin.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle used to locally identify the `DomainParticipant`.
    /// * `participant_security_config` - The [`ParticipantSecurityConfig`] configuration.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] if an error occurs. Otherwise, it shall return the [`ParticipantSecurityAlgorithmInfo`].
    fn set_participant_security_config(
        &mut self,
        handle: &Self::IdentityHandle,
        participant_security_config: &ParticipantSecurityConfig,
    ) -> Result<ParticipantSecurityAlgorithmInfo, SecurityException>;
}

impl Authentication for () {
    type IdentityHandle = ();
    type HandshakeHandle = ();
    type SharedSecretHandle = ();

    fn validate_local_identity(
        &mut self,
        _domain_id: DomainId,
        _participant_qos: &DomainParticipantQos,
        _candidate_participant_guid: Guid,
    ) -> Result<ValidateLocalIdentityOut<Self::IdentityHandle>, ValidationResult> {
        unreachable!("Placeholder should never be called")
    }

    fn validate_remote_identity(
        &mut self,
        _local_identity_handle: &Self::IdentityHandle,
        _remote_identity_token: IdentityToken,
        _remote_auth_request_token: Option<AuthRequestMessageToken>,
        _remote_participant_guid: Guid,
    ) -> Result<ValidateRemoteIdentityOut<Self::IdentityHandle>, ValidationResult> {
        unreachable!("Placeholder should never be called")
    }

    fn begin_handshake_request(
        &mut self,
        _initiator_identity_handle: &Self::IdentityHandle,
        _replier_identity_handle: &Self::IdentityHandle,
        _serialized_local_participant_data: &[u8],
    ) -> Result<BeginHandshakeRequestOut<Self::HandshakeHandle>, ValidationResult> {
        unreachable!("Placeholder should never be called")
    }

    fn begin_handshake_reply(
        &mut self,
        _handshake_message_in: HandshakeMessageToken,
        _initiator_identity_handle: &Self::IdentityHandle,
        _replier_identity_handle: &Self::IdentityHandle,
        _serialized_local_participant_data: &[u8],
    ) -> Result<BeginHandshakeReplyOut<Self::HandshakeHandle>, ValidationResult> {
        unreachable!("Placeholder should never be called")
    }

    fn process_handshake(
        &mut self,
        _handshake_message_in: HandshakeMessageToken,
        _handshake_handle: &Self::HandshakeHandle,
    ) -> Result<HandshakeMessageToken, ValidationResult> {
        unreachable!("Placeholder should never be called")
    }

    fn get_shared_secret(
        &mut self,
        _handshake_handle: &Self::HandshakeHandle,
    ) -> Result<Self::SharedSecretHandle, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn get_authenticated_peer_credential_token(
        &mut self,
        _handshake_handle: &Self::HandshakeHandle,
    ) -> Result<AuthenticatedPeerCredentialToken, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn get_identity_token(
        &mut self,
        _handle: &Self::IdentityHandle,
    ) -> Result<IdentityToken, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn get_identity_status_token(
        &mut self,
        _handle: &Self::IdentityHandle,
    ) -> Result<IdentityStatusToken, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn set_participant_security_config(
        &mut self,
        _handle: &Self::IdentityHandle,
        _participant_security_config: &ParticipantSecurityConfig,
    ) -> Result<ParticipantSecurityAlgorithmInfo, SecurityException> {
        unreachable!("Placeholder should never be called")
    }
}
