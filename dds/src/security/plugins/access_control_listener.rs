use super::types::SecurityException;

/// AccessControlListener plugin interface as defined in Section 9.4.2.10 of the DDS Security specification.
pub trait AccessControlListener: Send + 'static {
    /// Opaque handle representing internal permissions state.
    type PermissionsHandle;

    /// DomainParticipants' permissions can be revoked/changed. This listener provides a callback for permission revocation/changes.
    ///
    /// # Arguments
    ///
    /// * `handle` - A [`PermissionsHandle`](Self::PermissionsHandle) object that corresponds to the permissions of a DDS Participant whose permissions are being revoked.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] if an error occurs.
    fn on_revoke_permissions(
        &mut self,
        handle: &Self::PermissionsHandle,
    ) -> Result<(), SecurityException>;
}

impl AccessControlListener for () {
    type PermissionsHandle = ();

    fn on_revoke_permissions(
        &mut self,
        _handle: &Self::PermissionsHandle,
    ) -> Result<(), SecurityException> {
        Ok(())
    }
}
