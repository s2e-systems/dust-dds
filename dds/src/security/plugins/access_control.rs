use super::{authentication::Authentication, types::SecurityException};
use crate::{
    infrastructure::{
        domain::DomainId,
        qos::{DataReaderQos, DataWriterQos, DomainParticipantQos, TopicQos},
        qos_policy::{DataTagQosPolicy, PartitionQosPolicy},
    },
    security::types::{AuthenticatedPeerCredentialToken, PermissionsToken},
};

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

    /// Validates the permissions of the previously authenticated remote `DomainParticipant`, given the
    /// [`PermissionsToken`] object received via DDS discovery and the
    /// [`AuthenticatedPeerCredentialToken`] obtained as part of the authentication process.
    /// The operation returns a [`PermissionsHandle`](Self::PermissionsHandle) object, if successful.
    ///
    /// # Arguments
    ///
    /// * `auth_plugin` - The [`Authentication`] plugin, which validated the identity of the remote `DomainParticipant`.
    /// * `local_identity_handle` - The [`IdentityHandle`](Authentication::IdentityHandle) returned by the authentication plugin for the local `DomainParticipant`.
    /// * `remote_identity_handle` - The [`IdentityHandle`](Authentication::IdentityHandle) returned by a successful call to the `validate_remote_identity` operation on the `Authentication` plugin.
    /// * `remote_permissions_token` - The [`PermissionsToken`] of the remote `DomainParticipant` received via DDS discovery inside the `permissions_token` member of the `ParticipantBuiltinTopicData`.
    /// * `remote_credential_token` - The [`AuthenticatedPeerCredentialToken`] of the remote `DomainParticipant` returned by the operation `get_authenticated_peer_credential_token` on the `Authentication` plugin.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case permission validation fails.
    fn validate_remote_permissions<A>(
        &mut self,
        auth_plugin: &mut A,
        local_identity_handle: &A::IdentityHandle,
        remote_identity_handle: &A::IdentityHandle,
        remote_permissions_token: PermissionsToken,
        remote_credential_token: AuthenticatedPeerCredentialToken,
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

    /// Enforces the permissions of the local `DomainParticipant` when creating or updating a `DataWriter`.
    ///
    /// When the local `DomainParticipant` creates a `DataWriter` for `topic_name` with the specified
    /// [`DataWriterQos`] associated with the `data_tag`, its permissions must allow this.
    ///
    /// This operation shall also be called when the application calls the operation `set_qos()` on a
    /// `DataWriter` to check if the `DomainParticipant` has the permissions needed for the updated
    /// [`DataWriterQos`] configuration. If `check_create_datawriter` does not succeed, the `set_qos`
    /// operation shall fail with the `NOT_ALLOWED_BY_SECURITY` error.
    ///
    /// # Arguments
    ///
    /// * `permissions_handle` - The [`PermissionsHandle`](Self::PermissionsHandle) object associated with the local `DomainParticipant`.
    /// * `domain_id` - The DDS Domain Id of the local `DomainParticipant` to which the local `DataWriter` will belong.
    /// * `topic_name` - The topic name that the `DataWriter` is supposed to write.
    /// * `qos` - The [`DataWriterQos`] policies of the local `DataWriter`.
    /// * `partition` - The [`PartitionQosPolicy`] of the local `Publisher` to which the `DataWriter` will belong.
    /// * `data_tag` - The [`DataTagQosPolicy`] that the local `DataWriter` is requesting to be associated with its data.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case permission check fails.
    fn check_create_datawriter(
        &mut self,
        permissions_handle: &Self::PermissionsHandle,
        domain_id: DomainId,
        topic_name: &str,
        qos: &DataWriterQos,
        partition: &PartitionQosPolicy,
        data_tag: &DataTagQosPolicy,
    ) -> Result<(), SecurityException>;

    /// Enforces the permissions of the local `DomainParticipant` when creating or updating a `DataReader`.
    ///
    /// When the local `DomainParticipant` creates a `DataReader` for `topic_name` with the specified
    /// [`DataReaderQos`] associated with the `data_tag`, its permissions must allow this.
    ///
    /// This operation shall also be called when the application calls the operation `set_qos()` on a
    /// `DataReader` to check if the `DomainParticipant` has the permissions needed for the updated
    /// [`DataReaderQos`] configuration. If `check_create_datareader` does not succeed, the `set_qos`
    /// operation shall fail with the `NOT_ALLOWED_BY_SECURITY` error.
    ///
    /// # Arguments
    ///
    /// * `permissions_handle` - The [`PermissionsHandle`](Self::PermissionsHandle) object associated with the local `DomainParticipant`.
    /// * `domain_id` - The DDS Domain Id of the local `DomainParticipant` to which the local `DataReader` will belong.
    /// * `topic_name` - The topic name that the `DataReader` is supposed to read.
    /// * `qos` - The [`DataReaderQos`] policies of the local `DataReader`.
    /// * `partition` - The [`PartitionQosPolicy`] of the local `Subscriber` to which the `DataReader` will belong.
    /// * `data_tag` - The [`DataTagQosPolicy`] that the local `DataReader` is requesting read access to.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case permission check fails.
    fn check_create_datareader(
        &mut self,
        permissions_handle: &Self::PermissionsHandle,
        domain_id: DomainId,
        topic_name: &str,
        qos: &DataReaderQos,
        partition: &PartitionQosPolicy,
        data_tag: &DataTagQosPolicy,
    ) -> Result<(), SecurityException>;

    /// Enforces the permissions of the local `DomainParticipant` when creating or updating a `Topic`.
    ///
    /// When an entity of the local `DomainParticipant` creates a `Topic` with `topic_name` and [`TopicQos`] `qos`,
    /// its permissions must allow this.
    ///
    /// This operation shall also be called when the application calls the operation `set_qos()` on the `Topic` to
    /// check if the `DomainParticipant` has the permissions needed for the new Qos configuration. If `check_create_topic`
    /// does not succeed, the `set_qos` operation shall fail with the `NOT_ALLOWED_BY_SECURITY` error.
    ///
    /// # Arguments
    ///
    /// * `permissions_handle` - The [`PermissionsHandle`](Self::PermissionsHandle) object associated with the local `DomainParticipant`.
    /// * `domain_id` - The DDS Domain Id of the local `DomainParticipant` that creates the `Topic`.
    /// * `topic_name` - The topic name to be created.
    /// * `qos` - The [`TopicQos`] policies of the local `Topic`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case permission check fails.
    fn check_create_topic(
        &mut self,
        permissions_handle: &Self::PermissionsHandle,
        domain_id: DomainId,
        topic_name: &str,
        qos: &TopicQos,
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

    fn check_create_datawriter(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _domain_id: DomainId,
        _topic_name: &str,
        _qos: &DataWriterQos,
        _partition: &PartitionQosPolicy,
        _data_tag: &DataTagQosPolicy,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
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
        unreachable!("Placeholder should never be called")
    }

    fn check_create_topic(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _domain_id: DomainId,
        _topic_name: &str,
        _qos: &TopicQos,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }
}
