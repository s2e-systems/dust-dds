use super::types::SecurityException;
use crate::{
    infrastructure::{domain::DomainId, qos::DomainParticipantQos},
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

/// Authentication plugin interface as defined in Section 9.3.2 of the DDS Security specification.
pub trait Authentication: Send + Sync {
    /// Opaque handle representing internal authentication state as defined in Section 9.3.2.3 of the DDS Security specification.
    type IdentityHandle;

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
        &self,
        domain_id: DomainId,
        participant_qos: &DomainParticipantQos,
        candidate_participant_guid: Guid,
    ) -> Result<ValidateLocalIdentityOut<Self::IdentityHandle>, ValidationResult>;
}
