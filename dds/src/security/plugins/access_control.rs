use super::{authentication::Authentication, types::SecurityException};
use crate::{
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
    security::types::{
        AuthenticatedPeerCredentialToken, PermissionsCredentialToken, PermissionsToken,
    },
};

/// Output of [`check_remote_datareader`](AccessControl::check_remote_datareader).
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct CheckRemoteDataReaderOut {
    /// Indicates whether the permissions of the remote `DataReader` are restricted to relaying the information.
    pub relay_only: bool,
}

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

    /// Enforces the permissions of the local `DomainParticipant` when registering an instance on a `DataWriter`.
    ///
    /// In case the access control requires a finer granularity at the instance level, this operation enforces the
    /// permissions of the local `DataWriter`. The `key` identifies the instance being registered and permissions
    /// are checked to determine if registration of the specified instance is allowed.
    ///
    /// # Arguments
    ///
    /// * `permissions_handle` - The [`PermissionsHandle`](Self::PermissionsHandle) object associated with the local `DomainParticipant`.
    /// * `writer` - The [`PublicationBuiltinTopicData`] describing the `DataWriter` that registers the instance.
    /// * `key` - The [`InstanceHandle`] key of the instance for which the registration permissions are being checked.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case permission check fails.
    fn check_local_datawriter_register_instance(
        &mut self,
        permissions_handle: &Self::PermissionsHandle,
        writer: &PublicationBuiltinTopicData,
        key: &InstanceHandle,
    ) -> Result<(), SecurityException>;

    /// Enforces the permissions of the local `DomainParticipant` when disposing an instance on a `DataWriter`.
    ///
    /// In case the access control requires a finer granularity at the instance level, this operation enforces the
    /// permissions of the local `DataWriter`. The `key` has to match the permissions for disposing an instance.
    ///
    /// # Arguments
    ///
    /// * `permissions_handle` - The [`PermissionsHandle`](Self::PermissionsHandle) object associated with the local `DomainParticipant`.
    /// * `writer` - The [`PublicationBuiltinTopicData`] describing the `DataWriter` that disposes the instance.
    /// * `key` - The [`InstanceHandle`] key of the instance for which the disposal permissions are being checked.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case permission check fails.
    fn check_local_datawriter_dispose_instance(
        &mut self,
        permissions_handle: &Self::PermissionsHandle,
        writer: &PublicationBuiltinTopicData,
        key: &InstanceHandle,
    ) -> Result<(), SecurityException>;

    /// Enforces the permissions of the remote `DomainParticipant`.
    ///
    /// When the remote `DomainParticipant` is discovered, the `domain_id` and `DomainParticipantQoS` contained in
    /// `participant_data` are checked to verify that joining that DDS Domain and using that QoS is allowed by its permissions.
    ///
    /// This operation shall also be called whenever a `DomainParticipant` detects a QoS change for a different (peer) `DomainParticipant`
    /// that is matched with a local `DomainParticipant`.
    ///
    /// # Arguments
    ///
    /// * `permissions_handle` - The [`PermissionsHandle`](Self::PermissionsHandle) object associated with the remote `DomainParticipant`.
    /// * `domain_id` - The domain id where the remote `DomainParticipant` is about to be created.
    /// * `participant_data` - The [`ParticipantBuiltinTopicData`] object associated with the remote `DomainParticipant`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case permission check fails.
    fn check_remote_participant(
        &mut self,
        permissions_handle: &Self::PermissionsHandle,
        domain_id: DomainId,
        participant_data: &ParticipantBuiltinTopicData,
    ) -> Result<(), SecurityException>;

    /// Enforces the permissions of a remote `DomainParticipant` for a `DataWriter`.
    ///
    /// This operation shall be called by a `DomainParticipant` prior to matching a local `DataReader` belonging to that
    /// `DomainParticipant` with a `DataWriter` belonging to a different (peer) `DomainParticipant`.
    ///
    /// This operation shall also be called whenever a `DomainParticipant` detects a QoS change for a `DataWriter` belonging to
    /// a different (peer) `DomainParticipant` that is matched with a local `DataReader`.
    ///
    /// This operation verifies that the peer `DomainParticipant` has the permissions necessary to publish data on the DDS Topic
    /// using the `DataWriterQoS` that appears in `publication_data`.
    ///
    /// # Arguments
    ///
    /// * `permissions_handle` - The [`PermissionsHandle`](Self::PermissionsHandle) object associated with the remote `DomainParticipant`.
    /// * `domain_id` - The domain id of the `DomainParticipant` to which the remote `DataWriter` belongs.
    /// * `publication_data` - The [`PublicationBuiltinTopicData`] object associated with the remote `DataWriter`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case permission check fails.
    fn check_remote_datawriter(
        &mut self,
        permissions_handle: &Self::PermissionsHandle,
        domain_id: DomainId,
        publication_data: &PublicationBuiltinTopicData,
    ) -> Result<(), SecurityException>;

    /// Enforces the permissions of a remote `DomainParticipant` for a `DataReader`.
    ///
    /// This operation shall be called by a `DomainParticipant` prior to matching a local `DataWriter` belonging to that
    /// `DomainParticipant` with a `DataReader` belonging to a different (peer) `DomainParticipant`.
    ///
    /// This operation shall also be called whenever a `DomainParticipant` detects a QoS change for a `DataReader` belonging to
    /// a different (peer) `DomainParticipant` that is matched with a local `DataWriter`.
    ///
    /// This operation verifies that the peer `DomainParticipant` has the permissions necessary to subscribe to data on the DDS Topic
    /// using the `DataReaderQoS` that appears in `subscription_data`.
    ///
    /// # Arguments
    ///
    /// * `permissions_handle` - The [`PermissionsHandle`](Self::PermissionsHandle) object associated with the remote `DomainParticipant`.
    /// * `domain_id` - The domain id of the `DomainParticipant` to which the remote `DataReader` belongs.
    /// * `subscription_data` - The [`SubscriptionBuiltinTopicData`] object associated with the remote `DataReader`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case permission check fails. On success, returns [`CheckRemoteDataReaderOut`].
    fn check_remote_datareader(
        &mut self,
        permissions_handle: &Self::PermissionsHandle,
        domain_id: DomainId,
        subscription_data: &SubscriptionBuiltinTopicData,
    ) -> Result<CheckRemoteDataReaderOut, SecurityException>;

    /// Enforces the permissions of the remote `DomainParticipant`.
    ///
    /// When the remote `DomainParticipant` creates a certain topic, the `topic_name` and optionally the `TopicQoS`
    /// extracted from `topic_data` are verified to ensure the remote `DomainParticipant` permissions allow it to create
    /// the DDS Topic with the specified QoS.
    ///
    /// # Arguments
    ///
    /// * `permissions_handle` - The [`PermissionsHandle`](Self::PermissionsHandle) object associated with the remote `DomainParticipant`.
    /// * `domain_id` - The DDS Domain Id of the `DomainParticipant`.
    /// * `topic_data` - The [`TopicBuiltinTopicData`] object associated with the `Topic`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case permission check fails.
    fn check_remote_topic(
        &mut self,
        permissions_handle: &Self::PermissionsHandle,
        domain_id: DomainId,
        topic_data: &TopicBuiltinTopicData,
    ) -> Result<(), SecurityException>;

    /// Enforces access control rules based on the `DataTag` associated with a local `DataWriter` and a matching `DataReader`.
    ///
    /// This operation shall be called for any local `DataWriter` that matches a `DataReader`.
    /// The operation shall be called after `check_create_datawriter` has been called on the local `DataWriter` and either
    /// `check_create_datareader` or `check_remote_datareader` has been called on the `DataReader`.
    ///
    /// This operation shall also be called when a local `DataWriter`, matched with a `DataReader`, detects a change on the
    /// QoS of the local `DataWriter` or the matched `DataReader`.
    ///
    /// # Arguments
    ///
    /// * `writer_permissions_handle` - The [`PermissionsHandle`](Self::PermissionsHandle) object associated with the `DomainParticipant` that contains the local `DataWriter`.
    /// * `reader_permissions_handle` - The [`PermissionsHandle`](Self::PermissionsHandle) object associated with the remote `DomainParticipant`.
    /// * `publication_data` - The [`PublicationBuiltinTopicData`] object associated with the local `DataWriter`.
    /// * `subscription_data` - The [`SubscriptionBuiltinTopicData`] object associated with the matched `DataReader`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case permission check fails.
    fn check_local_datawriter_match(
        &mut self,
        writer_permissions_handle: &Self::PermissionsHandle,
        reader_permissions_handle: &Self::PermissionsHandle,
        publication_data: &PublicationBuiltinTopicData,
        subscription_data: &SubscriptionBuiltinTopicData,
    ) -> Result<(), SecurityException>;

    /// Enforces access control rules based on the `DataTag` associated with a local `DataReader` and a matching `DataWriter`.
    ///
    /// This operation shall be called for any local `DataReader` that matches a `DataWriter`.
    /// The operation shall be called after `check_create_datareader` has been called on the local `DataReader` and either
    /// `check_create_datawriter` or `check_remote_datawriter` has been called on the `DataWriter`.
    ///
    /// This operation shall also be called when a local `DataReader`, matched with a `DataWriter`, detects a change on the
    /// QoS of the local `DataReader` or the matched `DataWriter`.
    ///
    /// # Arguments
    ///
    /// * `reader_permissions_handle` - The [`PermissionsHandle`](Self::PermissionsHandle) object associated with the local `DomainParticipant` that contains the local `DataReader`.
    /// * `writer_permissions_handle` - The [`PermissionsHandle`](Self::PermissionsHandle) object associated with the remote `DomainParticipant`.
    /// * `subscription_data` - The [`SubscriptionBuiltinTopicData`] object associated with the local `DataReader`.
    /// * `publication_data` - The [`PublicationBuiltinTopicData`] object associated with the matched `DataWriter`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case permission check fails.
    fn check_local_datareader_match(
        &mut self,
        reader_permissions_handle: &Self::PermissionsHandle,
        writer_permissions_handle: &Self::PermissionsHandle,
        subscription_data: &SubscriptionBuiltinTopicData,
        publication_data: &PublicationBuiltinTopicData,
    ) -> Result<(), SecurityException>;

    /// Enforces the permissions of the remote `DomainParticipant` when registering an instance on a remote `DataWriter`.
    ///
    /// In case the access control requires a finer granularity at the instance level, this operation enforces the
    /// permissions of the remote `DataWriter`. The `key` has to match the permissions for registering an instance.
    ///
    /// # Arguments
    ///
    /// * `permissions_handle` - The [`PermissionsHandle`](Self::PermissionsHandle) object associated with the remote `DomainParticipant`.
    /// * `reader` - The [`SubscriptionBuiltinTopicData`] describing the local `DataReader` that is matched to the remote `DataWriter`.
    /// * `publication_handle` - The [`InstanceHandle`] that identifies the remote `DataWriter`.
    /// * `key` - The [`InstanceHandle`] key of the instance that needs to match the permissions for registering an instance.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case permission check fails.
    fn check_remote_datawriter_register_instance(
        &mut self,
        permissions_handle: &Self::PermissionsHandle,
        reader: &SubscriptionBuiltinTopicData,
        publication_handle: &InstanceHandle,
        key: &InstanceHandle,
    ) -> Result<(), SecurityException>;

    /// Enforces the permissions of the remote `DomainParticipant` when disposing an instance on a remote `DataWriter`.
    ///
    /// In case the access control requires a finer granularity at the instance level, this operation enforces the
    /// permissions of the remote `DataWriter`. The `key` has to match the permissions for disposing an instance.
    ///
    /// # Arguments
    ///
    /// * `permissions_handle` - The [`PermissionsHandle`](Self::PermissionsHandle) object associated with the remote `DomainParticipant`.
    /// * `reader` - The [`SubscriptionBuiltinTopicData`] describing the local `DataReader` that is matched to the remote `DataWriter` (Publication).
    /// * `publication_handle` - The [`InstanceHandle`] that identifies the remote `DataWriter` (Publication).
    /// * `key` - The [`InstanceHandle`] key of the instance that needs to match the permissions for disposing an instance.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case permission check fails.
    fn check_remote_datawriter_dispose_instance(
        &mut self,
        permissions_handle: &Self::PermissionsHandle,
        reader: &SubscriptionBuiltinTopicData,
        publication_handle: &InstanceHandle,
        key: &InstanceHandle,
    ) -> Result<(), SecurityException>;

    /// Retrieves a [`PermissionsToken`] object propagated via DDS discovery to summarize the permissions of the `DomainParticipant` identified by `handle`.
    ///
    /// # Arguments
    ///
    /// * `handle` - The [`PermissionsHandle`](Self::PermissionsHandle) used to locally identify the permissions of the `DomainParticipant`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case retrieving the permissions token fails.
    fn get_permissions_token(
        &mut self,
        handle: &Self::PermissionsHandle,
    ) -> Result<PermissionsToken, SecurityException>;

    /// Retrieves a [`PermissionsCredentialToken`] object that can be used to represent on the network the permissions of the `DomainParticipant` identified by `handle`.
    ///
    /// # Arguments
    ///
    /// * `handle` - The [`PermissionsHandle`](Self::PermissionsHandle) used to locally identify the permissions of the `DomainParticipant`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case retrieving the permissions credential token fails.
    fn get_permissions_credential_token(
        &mut self,
        handle: &Self::PermissionsHandle,
    ) -> Result<PermissionsCredentialToken, SecurityException>;
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

    fn check_local_datawriter_register_instance(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _writer: &PublicationBuiltinTopicData,
        _key: &InstanceHandle,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn check_local_datawriter_dispose_instance(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _writer: &PublicationBuiltinTopicData,
        _key: &InstanceHandle,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn check_remote_participant(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _domain_id: DomainId,
        _participant_data: &ParticipantBuiltinTopicData,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn check_remote_datawriter(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _domain_id: DomainId,
        _publication_data: &PublicationBuiltinTopicData,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn check_remote_datareader(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _domain_id: DomainId,
        _subscription_data: &SubscriptionBuiltinTopicData,
    ) -> Result<CheckRemoteDataReaderOut, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn check_remote_topic(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _domain_id: DomainId,
        _topic_data: &TopicBuiltinTopicData,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn check_local_datawriter_match(
        &mut self,
        _writer_permissions_handle: &Self::PermissionsHandle,
        _reader_permissions_handle: &Self::PermissionsHandle,
        _publication_data: &PublicationBuiltinTopicData,
        _subscription_data: &SubscriptionBuiltinTopicData,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn check_local_datareader_match(
        &mut self,
        _reader_permissions_handle: &Self::PermissionsHandle,
        _writer_permissions_handle: &Self::PermissionsHandle,
        _subscription_data: &SubscriptionBuiltinTopicData,
        _publication_data: &PublicationBuiltinTopicData,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn check_remote_datawriter_register_instance(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _reader: &SubscriptionBuiltinTopicData,
        _publication_handle: &InstanceHandle,
        _key: &InstanceHandle,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn check_remote_datawriter_dispose_instance(
        &mut self,
        _permissions_handle: &Self::PermissionsHandle,
        _reader: &SubscriptionBuiltinTopicData,
        _publication_handle: &InstanceHandle,
        _key: &InstanceHandle,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn get_permissions_token(
        &mut self,
        _handle: &Self::PermissionsHandle,
    ) -> Result<PermissionsToken, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn get_permissions_credential_token(
        &mut self,
        _handle: &Self::PermissionsHandle,
    ) -> Result<PermissionsCredentialToken, SecurityException> {
        unreachable!("Placeholder should never be called")
    }
}
