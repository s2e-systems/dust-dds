use super::{authentication::Authentication, types::SecurityException};
use crate::infrastructure::{domain::DomainId, qos::DomainParticipantQos};

/// AccessControl plugin interface as defined in Section 9.4.2 of the DDS Security specification.
pub trait AccessControl: Send + Sync {
    /// Opaque handle representing internal authentication state as defined in Section 9.3.2.3 of the DDS Security specification.
    type IdentityHandle;

    /// Opaque handle representing internal permissions state as defined in Section 9.4.2.3 of the DDS Security specification.
    type PermissionsHandle;

    /// Validates the permissions of the local `DomainParticipant`.
    ///
    /// The operation returns a [`PermissionsHandle`] object, if successful. The [`PermissionsHandle`] can be used to locally
    /// identify the permissions of the local `DomainParticipant` to the `AccessControl` plugin.
    ///
    /// This operation shall be called before the `DomainParticipant` is enabled. It shall be called either
    /// by the implementation of `DomainParticipantFactory::create_domain_participant` or
    /// `DomainParticipant::enable`.
    ///
    /// # Arguments
    ///
    /// * `auth_plugin` - The [`Authentication`] plugin, which validated the identity of the local `DomainParticipant`.
    /// * `identity` - The [`IdentityHandle`] returned by the authentication plugin from a successful call to `validate_local_identity`.
    /// * `domain_id` - The DDS Domain Id of the `DomainParticipant`.
    /// * `participant_qos` - The [`DomainParticipantQos`] of the `DomainParticipant`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case permission validation fails.
    fn validate_local_permissions(
        &self,
        auth_plugin: &dyn Authentication<IdentityHandle = Self::IdentityHandle>,
        identity: Self::IdentityHandle,
        domain_id: DomainId,
        participant_qos: &DomainParticipantQos,
    ) -> Result<Self::PermissionsHandle, SecurityException>;
}
