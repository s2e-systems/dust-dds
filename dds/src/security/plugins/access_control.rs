use super::{authentication::Authentication, types::SecurityException};
use crate::infrastructure::{domain::DomainId, qos::DomainParticipantQos};

/// AccessControl plugin interface as defined in Section 9.4.2 of the DDS Security specification.
pub trait AccessControl: Send + 'static {
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
    fn validate_local_permissions<A>(
        &mut self,
        auth_plugin: &mut A,
        identity: &A::IdentityHandle,
        domain_id: DomainId,
        participant_qos: &DomainParticipantQos,
    ) -> Result<Self::PermissionsHandle, SecurityException>
    where
        A: Authentication;

    /// Enforces the permissions of the local `DomainParticipant`.
    ///
    /// When the local `DomainParticipant` is created, its permissions must allow it to join the DDS Domain specified
    /// by the `domain_id`. Optionally the use of the specified value for the [`DomainParticipantQos`] must
    /// also be allowed by its permissions.
    ///
    /// This operation shall be called before the `DomainParticipant` is enabled. It shall be called either
    /// by the implementation of `DomainParticipantFactory::create_domain_participant` or
    /// `DomainParticipant::enable`.
    ///
    /// This operation shall also be called when the application calls the operation `set_qos()` on the `DomainParticipant` to
    /// check if the `DomainParticipant` has the permissions needed for the updated
    /// [`DomainParticipantQos`] configuration. The check performed shall be the same as the one
    /// performed when the `DomainParticipant` is first created, but using the new Qos specified in the
    /// `set_qos()`. If the `check_create_participant` does not succeed, the `set_qos`
    /// operation shall fail with the `NOT_ALLOWED_BY_SECURITY` error.
    ///
    /// # Arguments
    ///
    /// * `permissions_handle` - The [`PermissionsHandle`](Self::PermissionsHandle) object associated with the local `DomainParticipant`.
    /// * `domain_id` - The domain id where the local `DomainParticipant` is about to be created.
    /// * `qos` - The [`DomainParticipantQos`] of the local `DomainParticipant`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case permission check fails.
    fn check_create_participant(
        &mut self,
        permissions_handle: &Self::PermissionsHandle,
        domain_id: DomainId,
        qos: &DomainParticipantQos,
    ) -> Result<(), SecurityException>;
}

impl AccessControl for () {
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
        unreachable!("Placeholder should never be called")
    }

    fn check_create_participant(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _domain_id: DomainId,
        _qos: &DomainParticipantQos,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }
}
