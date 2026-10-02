use super::{
    access_control::AccessControl, authentication::Authentication, types::SecurityException,
};
use crate::{
    rtps::types::Property,
    security::types::{ParticipantSecurityAlgorithmInfo, ParticipantSecurityConfig},
};

/// Output of [`register_local_participant`](CryptoKeyFactory::register_local_participant).
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct RegisterLocalParticipantOut<ParticipantCryptoHandle> {
    /// Participant crypto handle.
    pub participant_crypto_handle: ParticipantCryptoHandle,
    /// Adjusted participant security algorithm info.
    pub adjusted_algorithm_info: ParticipantSecurityAlgorithmInfo,
}

/// CryptoKeyFactory plugin interface as defined in Section 9.5.2 of the DDS Security specification.
pub trait CryptoKeyFactory: Send + 'static {
    /// Opaque handle representing internal cryptographic participant state.
    type ParticipantCryptoHandle;

    /// Registers a local `DomainParticipant` with the Cryptographic Plugin.
    ///
    /// The `DomainParticipant` must have been already authenticated and granted access to the DDS Domain.
    /// The operation shall create any necessary key material that is needed to Encrypt and Sign secure messages
    /// that are directed to other `DomainParticipant` entities on the DDS Domain.
    ///
    /// # Arguments
    ///
    /// * `auth_plugin` - The [`Authentication`] plugin.
    /// * `access_control_plugin` - The [`AccessControl`] plugin.
    /// * `participant_identity` - An [`IdentityHandle`](Authentication::IdentityHandle) returned by a prior call to `validate_local_identity`.
    /// * `participant_permissions` - A [`PermissionsHandle`](AccessControl::PermissionsHandle) returned by a prior call to `validate_local_permissions`.
    /// * `participant_properties` - Properties from the `PropertyQosPolicy` of the local `DomainParticipant` whose name has the prefix `"dds.sec.crypto."`.
    /// * `participant_security_config` - The [`ParticipantSecurityConfig`] returned by `get_participant_security_config` on `AccessControl`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case registration fails.
    fn register_local_participant<A, C>(
        &mut self,
        auth_plugin: &mut A,
        access_control_plugin: &mut C,
        participant_identity: &A::IdentityHandle,
        participant_permissions: &C::PermissionsHandle,
        participant_properties: &[Property],
        participant_security_config: &ParticipantSecurityConfig,
    ) -> Result<RegisterLocalParticipantOut<Self::ParticipantCryptoHandle>, SecurityException>
    where
        A: Authentication,
        C: AccessControl;
}

impl CryptoKeyFactory for () {
    type ParticipantCryptoHandle = ();
    fn register_local_participant<A, C>(
        &mut self,
        auth_plugin: &mut A,
        access_control_plugin: &mut C,
        participant_identity: &A::IdentityHandle,
        participant_permissions: &C::PermissionsHandle,
        participant_properties: &[Property],
        participant_security_config: &ParticipantSecurityConfig,
    ) -> Result<RegisterLocalParticipantOut<Self::ParticipantCryptoHandle>, SecurityException>
    where
        A: Authentication,
        C: AccessControl,
    {
        unreachable!("Placeholder should never be called")
    }
}
