use super::types::SecurityException;

/// Enumerates the kind of changes to the status of the Authentication plugin or underlying Identity as defined in Section 9.3.2.12.1 of the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AuthStatusKind {
    /// Indicates a change to an identity status.
    IdentityStatus,
}

/// AuthenticationListener plugin interface as defined in Section 9.3.2.12 of the DDS Security specification.
pub trait AuthenticationListener: Send + 'static {
    /// Opaque handle representing internal authentication state.
    type IdentityHandle;

    /// Revokes the identity of the participant identified by the `IdentityHandle`.
    ///
    /// # Arguments
    ///
    /// * `handle` - Handle corresponding to the Identity of a DDS Participant whose identity is being revoked.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] if an error occurs.
    fn on_revoke_identity(
        &mut self,
        handle: &Self::IdentityHandle,
    ) -> Result<(), SecurityException>;

    /// Informs the `DomainParticipant` that a status associated with the Authentication plugin, or an Identity managed by the plugin, has changed.
    ///
    /// # Arguments
    ///
    /// * `handle` - Handle corresponding to the Identity.
    /// * `status_kind` - The [`AuthStatusKind`] indicating the status change.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] if an error occurs.
    fn on_status_changed(
        &mut self,
        handle: &Self::IdentityHandle,
        status_kind: AuthStatusKind,
    ) -> Result<(), SecurityException>;
}

impl AuthenticationListener for () {
    type IdentityHandle = ();

    fn on_revoke_identity(
        &mut self,
        _handle: &Self::IdentityHandle,
    ) -> Result<(), SecurityException> {
        Ok(())
    }

    fn on_status_changed(
        &mut self,
        _handle: &Self::IdentityHandle,
        _status_kind: AuthStatusKind,
    ) -> Result<(), SecurityException> {
        Ok(())
    }
}
