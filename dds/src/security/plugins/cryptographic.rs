use super::{
    access_control::AccessControl, authentication::Authentication, types::SecurityException,
};
use crate::{
    rtps::types::Property,
    security::types::{
        CryptoTokenSeq, CryptoTransformKeyRevisionIntHolder, DatareaderCryptoTokenSeq,
        DatawriterCryptoTokenSeq, EndpointSecurityAlgorithmInfo, EndpointSecurityConfig,
        ParticipantCryptoTokenSeq, ParticipantSecurityAlgorithmInfo, ParticipantSecurityConfig,
        SecureSubmessageCategory,
    },
};

/// Output of [`register_local_participant`](CryptoKeyFactory::register_local_participant).
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct RegisterLocalParticipantOut<ParticipantCryptoHandle> {
    /// Participant crypto handle.
    pub participant_crypto_handle: ParticipantCryptoHandle,
    /// Adjusted participant security algorithm info.
    pub adjusted_algorithm_info: ParticipantSecurityAlgorithmInfo,
}

/// Output of [`register_local_datawriter`](CryptoKeyFactory::register_local_datawriter).
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct RegisterLocalDatawriterOut<DatawriterCryptoHandle> {
    /// DataWriter crypto handle.
    pub datawriter_crypto_handle: DatawriterCryptoHandle,
    /// Adjusted endpoint security algorithm info.
    pub adjusted_algorithm_info: EndpointSecurityAlgorithmInfo,
}

/// Output of [`register_local_datareader`](CryptoKeyFactory::register_local_datareader).
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct RegisterLocalDatareaderOut<DatareaderCryptoHandle> {
    /// DataReader crypto handle.
    pub datareader_crypto_handle: DatareaderCryptoHandle,
    /// Adjusted endpoint security algorithm info.
    pub adjusted_algorithm_info: EndpointSecurityAlgorithmInfo,
}

/// Cryptographic plugin interface as defined in Section 9.5.1 of the DDS Security specification.
///
/// Combines [`CryptoKeyFactory`], [`CryptoKeyExchange`], and [`CryptoTransform`].
pub trait Cryptographic: CryptoKeyFactory + CryptoKeyExchange + CryptoTransform {}

impl Cryptographic for () {}

/// CryptoKeyFactory plugin interface as defined in Section 9.5.2 of the DDS Security specification.
pub trait CryptoKeyFactory: Send + 'static {
    /// Opaque handle representing internal cryptographic participant state.
    type ParticipantCryptoHandle;

    /// Opaque handle representing internal cryptographic DataWriter state.
    type DatawriterCryptoHandle;

    /// Opaque handle representing internal cryptographic DataReader state.
    type DatareaderCryptoHandle;

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
    fn register_local_participant<'a, A, C>(
        &mut self,
        auth_plugin: &mut A,
        access_control_plugin: &mut C,
        participant_identity: &A::IdentityHandle,
        participant_permissions: &C::PermissionsHandle,
        participant_properties: impl Iterator<Item = &'a Property>,
        participant_security_config: &ParticipantSecurityConfig,
    ) -> Result<RegisterLocalParticipantOut<Self::ParticipantCryptoHandle>, SecurityException>
    where
        A: Authentication,
        C: AccessControl;

    /// Registers a remote `DomainParticipant` with the Cryptographic Plugin.
    ///
    /// The remote `DomainParticipant` must have been already Authenticated and granted Access to the DDS Domain.
    /// The operation performs two functions:
    /// 1. It shall create any necessary key material needed to decrypt and verify the signatures of
    ///    messages received from that remote `DomainParticipant` and directed to the local `DomainParticipant`.
    /// 2. It shall create any necessary key material that will be used by the local `DomainParticipant`
    ///    when encrypting or signing messages that are intended only for that remote `DomainParticipant`.
    ///
    /// # Arguments
    ///
    /// * `local_participant_crypto_handle` - A [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) returned by a prior call to `register_local_participant`.
    /// * `remote_participant_identity` - An [`IdentityHandle`](Authentication::IdentityHandle) returned by a prior call to `validate_remote_identity`.
    /// * `remote_participant_permissions` - A [`PermissionsHandle`](AccessControl::PermissionsHandle) returned by a prior call to `validate_remote_permissions`.
    /// * `shared_secret` - The [`SharedSecretHandle`](Authentication::SharedSecretHandle) returned by a prior call to `get_shared_secret`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case registration fails.
    fn register_matched_remote_participant<A, C>(
        &mut self,
        local_participant_crypto_handle: &Self::ParticipantCryptoHandle,
        remote_participant_identity: &A::IdentityHandle,
        remote_participant_permissions: &C::PermissionsHandle,
        shared_secret: &A::SharedSecretHandle,
    ) -> Result<Self::ParticipantCryptoHandle, SecurityException>
    where
        A: Authentication,
        C: AccessControl;

    /// Registers a local `DataWriter` with the Cryptographic Plugin.
    ///
    /// # Arguments
    ///
    /// * `participant_crypto` - A [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) returned by a prior call to `register_local_participant`.
    /// * `local_datawriter_properties` - Properties from the `PropertyQosPolicy` of the local `DataWriter` whose name has the prefix `"dds.sec.crypto."`.
    /// * `datawriter_security_config` - The [`EndpointSecurityConfig`] returned by `get_datawriter_security_config` on `AccessControl`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case registration fails.
    fn register_local_datawriter(
        &mut self,
        participant_crypto: &Self::ParticipantCryptoHandle,
        local_datawriter_properties: &[Property],
        datawriter_security_config: &EndpointSecurityConfig,
    ) -> Result<RegisterLocalDatawriterOut<Self::DatawriterCryptoHandle>, SecurityException>;

    /// Registers a remote `DataReader` with the Cryptographic Plugin.
    ///
    /// The remote `DataReader` shall correspond to one that has been granted permissions to match with the local `DataWriter`.
    /// This operation shall create the cryptographic material necessary to encrypt and/or sign the RTPS
    /// submessages (Data, DataFrag, Gap, Heartbeat, HeartbeatFrag) sent from the local `DataWriter` to that `DataReader`.
    /// It shall also create the cryptographic material necessary to process RTPS Submessages (AckNack, NackFrag) sent from the
    /// remote `DataReader` to the `DataWriter`.
    ///
    /// # Arguments
    ///
    /// * `local_datawriter_crypto_handle` - A [`DatawriterCryptoHandle`](Self::DatawriterCryptoHandle) returned by a prior call to `register_local_datawriter`.
    /// * `remote_participant_crypto` - A [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) returned by a prior call to `register_matched_remote_participant`.
    /// * `shared_secret` - The [`SharedSecretHandle`](Authentication::SharedSecretHandle) returned by a prior call to `get_shared_secret`.
    /// * `relay_only` - Boolean indicating whether the cryptographic material to be generated for the remote `DataReader` shall contain everything, or only the material necessary to relay the information.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case registration fails.
    fn register_matched_remote_datareader<A>(
        &mut self,
        local_datawriter_crypto_handle: &Self::DatawriterCryptoHandle,
        remote_participant_crypto: &Self::ParticipantCryptoHandle,
        shared_secret: &A::SharedSecretHandle,
        relay_only: bool,
    ) -> Result<Self::DatawriterCryptoHandle, SecurityException>
    where
        A: Authentication;

    /// Registers a local `DataReader` with the Cryptographic Plugin.
    ///
    /// # Arguments
    ///
    /// * `participant_crypto` - A [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) returned by a prior call to `register_local_participant`.
    /// * `local_datareader_properties` - Properties from the `PropertyQosPolicy` of the local `DataReader` whose name has the prefix `"dds.sec.crypto."`.
    /// * `datareader_security_config` - The [`EndpointSecurityConfig`] returned by `get_datareader_security_config` on `AccessControl`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case registration fails.
    fn register_local_datareader(
        &mut self,
        participant_crypto: &Self::ParticipantCryptoHandle,
        local_datareader_properties: &[Property],
        datareader_security_config: &EndpointSecurityConfig,
    ) -> Result<RegisterLocalDatareaderOut<Self::DatareaderCryptoHandle>, SecurityException>;

    /// Registers a remote `DataWriter` with the Cryptographic Plugin.
    ///
    /// The remote `DataWriter` shall correspond to one that has been granted permissions to match with the local `DataReader`.
    /// This operation shall create the cryptographic material necessary to decrypt and/or verify the signatures
    /// of the RTPS submessages (Data, DataFrag, Heartbeat, HeartbeatFrag, Gap) sent from the remote `DataWriter` to the `DataReader`.
    /// The operation shall also create the cryptographic material necessary to encrypt and/or sign the RTPS submessages (AckNack, NackFrag) sent from the local
    /// `DataReader` to the remote `DataWriter`.
    ///
    /// # Arguments
    ///
    /// * `local_datareader_crypto_handle` - A [`DatareaderCryptoHandle`](Self::DatareaderCryptoHandle) returned by a prior call to `register_local_datareader`.
    /// * `remote_participant_crypto` - A [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) returned by a prior call to `register_matched_remote_participant`.
    /// * `shared_secret` - The [`SharedSecretHandle`](Authentication::SharedSecretHandle) returned by a prior call to `get_shared_secret`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case registration fails.
    fn register_matched_remote_datawriter<A>(
        &mut self,
        local_datareader_crypto_handle: &Self::DatareaderCryptoHandle,
        remote_participant_crypto: &Self::ParticipantCryptoHandle,
        shared_secret: &A::SharedSecretHandle,
    ) -> Result<Self::DatareaderCryptoHandle, SecurityException>
    where
        A: Authentication;

    /// Creates a revision of the KeyMaterial used by the local DomainParticipant and its contained
    /// DataReader and DataWriter entities.
    ///
    /// # Arguments
    ///
    /// * `participant_crypto_handle` - A [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) returned by a prior call to `register_local_participant`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case operation fails.
    fn revise_local_entity_keys(
        &mut self,
        participant_crypto_handle: &Self::ParticipantCryptoHandle,
    ) -> Result<CryptoTransformKeyRevisionIntHolder, SecurityException>;

    /// Configures the plugin to start using the KeyMaterial that corresponds to a Key Revision created by a
    /// previous call to the operation `revise_local_entity_keys`.
    ///
    /// # Arguments
    ///
    /// * `participant_crypto_handle` - A [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) returned by a prior call to `register_local_participant`.
    /// * `key_revision` - The [`CryptoTransformKeyRevisionIntHolder`] value returned by a prior call to `revise_local_entity_keys`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case operation fails.
    fn activate_key_revision(
        &mut self,
        participant_crypto_handle: &Self::ParticipantCryptoHandle,
        key_revision: CryptoTransformKeyRevisionIntHolder,
    ) -> Result<(), SecurityException>;

    /// Releases the resources associated with a `DomainParticipant` that the Cryptographic plugin maintains.
    ///
    /// # Arguments
    ///
    /// * `participant_crypto_handle` - A [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) returned by a prior call
    ///   to `register_local_participant` or `register_matched_remote_participant`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case unregistration fails.
    fn unregister_participant(
        &mut self,
        participant_crypto_handle: Self::ParticipantCryptoHandle,
    ) -> Result<(), SecurityException>;

    /// Releases the resources associated with a `DataWriter` that the Cryptographic plugin maintains.
    ///
    /// # Arguments
    ///
    /// * `datawriter_crypto_handle` - A [`DatawriterCryptoHandle`](Self::DatawriterCryptoHandle) returned by a prior call
    ///   to `register_local_datawriter` or `register_matched_remote_datareader`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case unregistration fails.
    fn unregister_datawriter(
        &mut self,
        datawriter_crypto_handle: Self::DatawriterCryptoHandle,
    ) -> Result<(), SecurityException>;

    /// Releases the resources associated with a `DataReader` that the Cryptographic plugin maintains.
    ///
    /// # Arguments
    ///
    /// * `datareader_crypto_handle` - A [`DatareaderCryptoHandle`](Self::DatareaderCryptoHandle) returned by a prior call
    ///   to `register_local_datareader` or `register_matched_remote_datawriter`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case unregistration fails.
    fn unregister_datareader(
        &mut self,
        datareader_crypto_handle: Self::DatareaderCryptoHandle,
    ) -> Result<(), SecurityException>;
}

impl CryptoKeyFactory for () {
    type ParticipantCryptoHandle = ();
    type DatawriterCryptoHandle = ();
    type DatareaderCryptoHandle = ();

    fn register_local_participant<'a, A, C>(
        &mut self,
        _auth_plugin: &mut A,
        _access_control_plugin: &mut C,
        _participant_identity: &A::IdentityHandle,
        _participant_permissions: &C::PermissionsHandle,
        _participant_properties: impl Iterator<Item = &'a Property>,
        _participant_security_config: &ParticipantSecurityConfig,
    ) -> Result<RegisterLocalParticipantOut<Self::ParticipantCryptoHandle>, SecurityException>
    where
        A: Authentication,
        C: AccessControl,
    {
        unreachable!("Placeholder should never be called")
    }

    fn register_matched_remote_participant<A, C>(
        &mut self,
        _local_participant_crypto_handle: &Self::ParticipantCryptoHandle,
        _remote_participant_identity: &A::IdentityHandle,
        _remote_participant_permissions: &C::PermissionsHandle,
        _shared_secret: &A::SharedSecretHandle,
    ) -> Result<Self::ParticipantCryptoHandle, SecurityException>
    where
        A: Authentication,
        C: AccessControl,
    {
        unreachable!("Placeholder should never be called")
    }

    fn register_local_datawriter(
        &mut self,
        _participant_crypto: &Self::ParticipantCryptoHandle,
        _local_datawriter_properties: &[Property],
        _datawriter_security_config: &EndpointSecurityConfig,
    ) -> Result<RegisterLocalDatawriterOut<Self::DatawriterCryptoHandle>, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn register_matched_remote_datareader<A>(
        &mut self,
        _local_datawriter_crypto_handle: &Self::DatawriterCryptoHandle,
        _remote_participant_crypto: &Self::ParticipantCryptoHandle,
        _shared_secret: &A::SharedSecretHandle,
        _relay_only: bool,
    ) -> Result<Self::DatawriterCryptoHandle, SecurityException>
    where
        A: Authentication,
    {
        unreachable!("Placeholder should never be called")
    }

    fn register_local_datareader(
        &mut self,
        _participant_crypto: &Self::ParticipantCryptoHandle,
        _local_datareader_properties: &[Property],
        _datareader_security_config: &EndpointSecurityConfig,
    ) -> Result<RegisterLocalDatareaderOut<Self::DatareaderCryptoHandle>, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn register_matched_remote_datawriter<A>(
        &mut self,
        _local_datareader_crypto_handle: &Self::DatareaderCryptoHandle,
        _remote_participant_crypto: &Self::ParticipantCryptoHandle,
        _shared_secret: &A::SharedSecretHandle,
    ) -> Result<Self::DatareaderCryptoHandle, SecurityException>
    where
        A: Authentication,
    {
        unreachable!("Placeholder should never be called")
    }

    fn revise_local_entity_keys(
        &mut self,
        _participant_crypto_handle: &Self::ParticipantCryptoHandle,
    ) -> Result<CryptoTransformKeyRevisionIntHolder, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn activate_key_revision(
        &mut self,
        _participant_crypto_handle: &Self::ParticipantCryptoHandle,
        _key_revision: CryptoTransformKeyRevisionIntHolder,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn unregister_participant(
        &mut self,
        _participant_crypto_handle: Self::ParticipantCryptoHandle,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn unregister_datawriter(
        &mut self,
        _datawriter_crypto_handle: Self::DatawriterCryptoHandle,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn unregister_datareader(
        &mut self,
        _datareader_crypto_handle: Self::DatareaderCryptoHandle,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }
}

/// CryptoKeyExchange plugin interface as defined in Section 9.5.3 of the DDS Security specification.
pub trait CryptoKeyExchange: Send + 'static {
    /// Opaque handle representing internal cryptographic participant state.
    type ParticipantCryptoHandle;

    /// Opaque handle representing internal cryptographic DataWriter state.
    type DatawriterCryptoHandle;

    /// Opaque handle representing internal cryptographic DataReader state.
    type DatareaderCryptoHandle;

    /// Creates a sequence of [`ParticipantCryptoTokenSeq`] tokens containing the information needed to
    /// correctly interpret ciphertext encoded using the `local_participant_crypto`.
    ///
    /// # Arguments
    ///
    /// * `local_participant_crypto` - A [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) returned by `register_local_participant`.
    /// * `remote_participant_crypto` - A [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) returned by `register_matched_remote_participant`.
    /// * `key_revision` - The key revision integer selecting the revision of the Key Material.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case operation fails.
    fn create_local_participant_crypto_tokens(
        &mut self,
        local_participant_crypto: &Self::ParticipantCryptoHandle,
        remote_participant_crypto: &Self::ParticipantCryptoHandle,
        key_revision: CryptoTransformKeyRevisionIntHolder,
    ) -> Result<ParticipantCryptoTokenSeq, SecurityException>;

    /// Configures the Cryptographic plugin with the key material necessary to interpret messages encoded by the remote `DomainParticipant`.
    ///
    /// # Arguments
    ///
    /// * `local_participant_crypto` - A [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) returned by `register_local_participant`.
    /// * `remote_participant_crypto` - A [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) returned by `register_matched_remote_participant`.
    /// * `remote_participant_tokens` - A [`ParticipantCryptoTokenSeq`] received via the `BuiltinParticipantVolatileMessageSecureReader`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case operation fails.
    fn set_remote_participant_crypto_tokens(
        &mut self,
        local_participant_crypto: &Self::ParticipantCryptoHandle,
        remote_participant_crypto: &Self::ParticipantCryptoHandle,
        remote_participant_tokens: ParticipantCryptoTokenSeq,
    ) -> Result<(), SecurityException>;

    /// Creates a sequence of [`DatawriterCryptoTokenSeq`] tokens containing the information needed to
    /// correctly interpret ciphertext encoded using the `local_datawriter_crypto`.
    ///
    /// # Arguments
    ///
    /// * `local_datawriter_crypto` - A [`DatawriterCryptoHandle`](Self::DatawriterCryptoHandle) returned by `register_local_datawriter`.
    /// * `remote_datareader_crypto` - A [`DatareaderCryptoHandle`](Self::DatareaderCryptoHandle) returned by `register_matched_remote_datareader`.
    /// * `key_revision` - The key revision integer selecting the revision of the Key Material.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case operation fails.
    fn create_local_datawriter_crypto_tokens(
        &mut self,
        local_datawriter_crypto: &Self::DatawriterCryptoHandle,
        remote_datareader_crypto: &Self::DatareaderCryptoHandle,
        key_revision: CryptoTransformKeyRevisionIntHolder,
    ) -> Result<DatawriterCryptoTokenSeq, SecurityException>;

    /// Configures the Cryptographic plugin with the key material necessary to interpret messages encoded by the remote `DataWriter`.
    ///
    /// # Arguments
    ///
    /// * `remote_datawriter_crypto` - A [`DatawriterCryptoHandle`](Self::DatawriterCryptoHandle) returned by `register_matched_remote_datawriter`.
    /// * `local_datareader_crypto` - A [`DatareaderCryptoHandle`](Self::DatareaderCryptoHandle) returned by `register_local_datareader`.
    /// * `remote_datawriter_tokens` - A [`DatawriterCryptoTokenSeq`] received via the `BuiltinParticipantVolatileMessageSecureReader`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case operation fails.
    fn set_remote_datawriter_crypto_tokens(
        &mut self,
        remote_datawriter_crypto: &Self::DatawriterCryptoHandle,
        local_datareader_crypto: &Self::DatareaderCryptoHandle,
        remote_datawriter_tokens: DatawriterCryptoTokenSeq,
    ) -> Result<(), SecurityException>;

    /// Creates a sequence of [`DatareaderCryptoTokenSeq`] tokens containing the information needed to
    /// correctly interpret ciphertext encoded using the `local_datareader_crypto`.
    ///
    /// # Arguments
    ///
    /// * `local_datareader_crypto` - A [`DatareaderCryptoHandle`](Self::DatareaderCryptoHandle) returned by `register_local_datareader`.
    /// * `remote_datawriter_crypto` - A [`DatawriterCryptoHandle`](Self::DatawriterCryptoHandle) returned by `register_matched_remote_datawriter`.
    /// * `key_revision` - The key revision integer selecting the revision of the Key Material.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case operation fails.
    fn create_local_datareader_crypto_tokens(
        &mut self,
        local_datareader_crypto: &Self::DatareaderCryptoHandle,
        remote_datawriter_crypto: &Self::DatawriterCryptoHandle,
        key_revision: CryptoTransformKeyRevisionIntHolder,
    ) -> Result<DatareaderCryptoTokenSeq, SecurityException>;

    /// Configures the Cryptographic plugin with the key material necessary to interpret messages encoded by the remote `DataReader`.
    ///
    /// # Arguments
    ///
    /// * `remote_datareader_crypto` - A [`DatareaderCryptoHandle`](Self::DatareaderCryptoHandle) returned by `register_matched_remote_datareader`.
    /// * `local_datawriter_crypto` - A [`DatawriterCryptoHandle`](Self::DatawriterCryptoHandle) returned by `register_local_datawriter`.
    /// * `remote_datareader_tokens` - A [`DatareaderCryptoTokenSeq`] received via the `BuiltinParticipantVolatileMessageSecureReader`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case operation fails.
    fn set_remote_datareader_crypto_tokens(
        &mut self,
        remote_datareader_crypto: &Self::DatareaderCryptoHandle,
        local_datawriter_crypto: &Self::DatawriterCryptoHandle,
        remote_datareader_tokens: DatareaderCryptoTokenSeq,
    ) -> Result<(), SecurityException>;

    /// Returns the tokens in the [`CryptoTokenSeq`] sequence to the plugin so it can release any information associated with it.
    ///
    /// # Arguments
    ///
    /// * `crypto_tokens` - A [`CryptoTokenSeq`] issued by a prior call to `create_local_participant_crypto_tokens`,
    ///   `create_local_datawriter_crypto_tokens`, or `create_local_datareader_crypto_tokens`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case operation fails.
    fn return_crypto_tokens(
        &mut self,
        crypto_tokens: CryptoTokenSeq,
    ) -> Result<(), SecurityException>;
}

impl CryptoKeyExchange for () {
    type ParticipantCryptoHandle = ();
    type DatawriterCryptoHandle = ();
    type DatareaderCryptoHandle = ();

    fn create_local_participant_crypto_tokens(
        &mut self,
        _local_participant_crypto: &Self::ParticipantCryptoHandle,
        _remote_participant_crypto: &Self::ParticipantCryptoHandle,
        _key_revision: CryptoTransformKeyRevisionIntHolder,
    ) -> Result<ParticipantCryptoTokenSeq, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn set_remote_participant_crypto_tokens(
        &mut self,
        _local_participant_crypto: &Self::ParticipantCryptoHandle,
        _remote_participant_crypto: &Self::ParticipantCryptoHandle,
        _remote_participant_tokens: ParticipantCryptoTokenSeq,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn create_local_datawriter_crypto_tokens(
        &mut self,
        _local_datawriter_crypto: &Self::DatawriterCryptoHandle,
        _remote_datareader_crypto: &Self::DatareaderCryptoHandle,
        _key_revision: CryptoTransformKeyRevisionIntHolder,
    ) -> Result<DatawriterCryptoTokenSeq, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn set_remote_datawriter_crypto_tokens(
        &mut self,
        _remote_datawriter_crypto: &Self::DatawriterCryptoHandle,
        _local_datareader_crypto: &Self::DatareaderCryptoHandle,
        _remote_datawriter_tokens: DatawriterCryptoTokenSeq,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn create_local_datareader_crypto_tokens(
        &mut self,
        _local_datareader_crypto: &Self::DatareaderCryptoHandle,
        _remote_datawriter_crypto: &Self::DatawriterCryptoHandle,
        _key_revision: CryptoTransformKeyRevisionIntHolder,
    ) -> Result<DatareaderCryptoTokenSeq, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn set_remote_datareader_crypto_tokens(
        &mut self,
        _remote_datareader_crypto: &Self::DatareaderCryptoHandle,
        _local_datawriter_crypto: &Self::DatawriterCryptoHandle,
        _remote_datareader_tokens: DatareaderCryptoTokenSeq,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn return_crypto_tokens(
        &mut self,
        _crypto_tokens: CryptoTokenSeq,
    ) -> Result<(), SecurityException> {
        unreachable!("Placeholder should never be called")
    }
}

/// Output of [`encode_serialized_payload`](CryptoTransform::encode_serialized_payload).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct EncodeSerializedPayloadOut {
    /// Length of encoded buffer containing CryptoContent.
    pub encoded_buffer_len: usize,
    /// Length of extra inline QoS parameters.
    pub extra_inline_qos_len: usize,
}

/// Output of [`encode_datawriter_submessage`](CryptoTransform::encode_datawriter_submessage).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct EncodeDatawriterSubmessageOut {
    /// Length of encoded RTPS submessage.
    pub encoded_rtps_submessage_len: usize,
    /// Index to use in subsequent calls to `encode_datawriter_submessage`.
    pub receiving_datareader_crypto_list_index: usize,
}

/// Output of [`encode_rtps_message`](CryptoTransform::encode_rtps_message).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct EncodeRtpsMessageOut {
    /// Length of encoded RTPS message.
    pub encoded_rtps_message_len: usize,
    /// Index to use in subsequent calls to `encode_rtps_message`.
    pub receiving_participant_crypto_list_index: usize,
}

/// Output of [`preprocess_secure_submsg`](CryptoTransform::preprocess_secure_submsg).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PreprocessSecureSubmsgOut<DatawriterCryptoHandle, DatareaderCryptoHandle> {
    /// Category of the secure submessage.
    pub secure_submessage_category: SecureSubmessageCategory,
    /// DataWriter crypto handle.
    pub datawriter_crypto: DatawriterCryptoHandle,
    /// DataReader crypto handle.
    pub datareader_crypto: DatareaderCryptoHandle,
}

/// CryptoTransform plugin interface as defined in Section 9.5.4 of the DDS Security specification.
pub trait CryptoTransform: Send + 'static {
    /// Opaque handle representing internal cryptographic DataWriter state.
    type DatawriterCryptoHandle;

    /// Opaque handle representing internal cryptographic DataReader state.
    type DatareaderCryptoHandle;

    /// Opaque handle representing internal cryptographic participant state.
    type ParticipantCryptoHandle;

    /// Encodes a `SerializedPayload` submessage element.
    ///
    /// # Arguments
    ///
    /// * `plain_buffer` - The input containing the `SerializedPayload` RTPS submessage element.
    /// * `sending_datawriter_crypto` - The [`DatawriterCryptoHandle`](Self::DatawriterCryptoHandle) returned by a previous call to `register_local_datawriter`.
    /// * `encoded_buffer` - Output buffer for `CryptoContent`.
    /// * `extra_inline_qos` - Output buffer for extra inline QoS parameters.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case operation fails.
    fn encode_serialized_payload(
        &mut self,
        plain_buffer: &[u8],
        sending_datawriter_crypto: &Self::DatawriterCryptoHandle,
        encoded_buffer: &mut [u8],
        extra_inline_qos: &mut [u8],
    ) -> Result<EncodeSerializedPayloadOut, SecurityException>;

    /// Encodes a DataWriter RTPS submessage.
    ///
    /// # Arguments
    ///
    /// * `plain_rtps_submessage` - The input containing the RTPS submessage created by a `DataWriter`.
    /// * `sending_datawriter_crypto` - The [`DatawriterCryptoHandle`](Self::DatawriterCryptoHandle) of the `DataWriter`.
    /// * `receiving_datareader_crypto_list` - List of [`DatareaderCryptoHandle`](Self::DatareaderCryptoHandle) for target `DataReader` entities.
    /// * `receiving_datareader_crypto_list_index` - Index into `receiving_datareader_crypto_list`.
    /// * `encoded_rtps_submessage` - Output buffer for encoded RTPS submessage.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case operation fails.
    fn encode_datawriter_submessage(
        &mut self,
        plain_rtps_submessage: &[u8],
        sending_datawriter_crypto: &Self::DatawriterCryptoHandle,
        receiving_datareader_crypto_list: &[Self::DatareaderCryptoHandle],
        receiving_datareader_crypto_list_index: usize,
        encoded_rtps_submessage: &mut [u8],
    ) -> Result<EncodeDatawriterSubmessageOut, SecurityException>;

    /// Encodes a DataReader RTPS submessage.
    ///
    /// # Arguments
    ///
    /// * `plain_rtps_submessage` - The input containing the RTPS submessage created by a `DataReader`.
    /// * `sending_datareader_crypto` - The [`DatareaderCryptoHandle`](Self::DatareaderCryptoHandle) of the `DataReader`.
    /// * `receiving_datawriter_crypto_list` - List of [`DatawriterCryptoHandle`](Self::DatawriterCryptoHandle) for target `DataWriter` entities.
    /// * `encoded_rtps_submessage` - Output buffer for encoded RTPS submessage.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case operation fails.
    fn encode_datareader_submessage(
        &mut self,
        plain_rtps_submessage: &[u8],
        sending_datareader_crypto: &Self::DatareaderCryptoHandle,
        receiving_datawriter_crypto_list: &[Self::DatawriterCryptoHandle],
        encoded_rtps_submessage: &mut [u8],
    ) -> Result<usize, SecurityException>;

    /// Encodes an RTPS message prior to sending it on the wire.
    ///
    /// # Arguments
    ///
    /// * `plain_rtps_message` - The input containing the RTPS message to be sent.
    /// * `sending_participant_crypto` - The [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) of the local `DomainParticipant`.
    /// * `receiving_participant_crypto_list` - List of [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) of target remote participants.
    /// * `receiving_participant_crypto_list_index` - Index into `receiving_participant_crypto_list`.
    /// * `transform_with_psk` - Indicates whether to protect using pre-shared key.
    /// * `encoded_rtps_message` - Output buffer for encoded RTPS message.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case operation fails.
    fn encode_rtps_message(
        &mut self,
        plain_rtps_message: &[u8],
        sending_participant_crypto: &Self::ParticipantCryptoHandle,
        receiving_participant_crypto_list: &[Self::ParticipantCryptoHandle],
        receiving_participant_crypto_list_index: usize,
        transform_with_psk: bool,
        encoded_rtps_message: &mut [u8],
    ) -> Result<Option<EncodeRtpsMessageOut>, SecurityException>;

    /// Decodes an RTPS message received from the network.
    ///
    /// # Arguments
    ///
    /// * `encoded_rtps_message` - The input containing the encoded RTPS message received.
    /// * `receiving_participant_crypto` - The [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) of the local receiving `DomainParticipant`.
    /// * `sending_participant_crypto` - The [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) of the remote sending `DomainParticipant`.
    /// * `plain_rtps_message` - Output buffer for plain RTPS message.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case decoding fails.
    fn decode_rtps_message(
        &mut self,
        encoded_rtps_message: &[u8],
        receiving_participant_crypto: &Self::ParticipantCryptoHandle,
        sending_participant_crypto: &Self::ParticipantCryptoHandle,
        plain_rtps_message: &mut [u8],
    ) -> Result<usize, SecurityException>;

    /// Preprocesses a secure submessage received in an RTPS message.
    ///
    /// # Arguments
    ///
    /// * `encoded_rtps_submessage` - The input containing the received RTPS submessage.
    /// * `receiving_participant_crypto` - The [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) of the local receiving `DomainParticipant`.
    /// * `sending_participant_crypto` - The [`ParticipantCryptoHandle`](Self::ParticipantCryptoHandle) of the remote sending `DomainParticipant`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case operation fails.
    fn preprocess_secure_submsg(
        &mut self,
        encoded_rtps_submessage: &[u8],
        receiving_participant_crypto: &Self::ParticipantCryptoHandle,
        sending_participant_crypto: &Self::ParticipantCryptoHandle,
    ) -> Result<
        PreprocessSecureSubmsgOut<Self::DatawriterCryptoHandle, Self::DatareaderCryptoHandle>,
        SecurityException,
    >;

    /// Decodes a DataWriter RTPS submessage.
    ///
    /// # Arguments
    ///
    /// * `encoded_rtps_submessage` - The input containing the encoded RTPS submessages.
    /// * `receiving_datareader_crypto` - The [`DatareaderCryptoHandle`](Self::DatareaderCryptoHandle) of the receiving `DataReader`.
    /// * `sending_datawriter_crypto` - The [`DatawriterCryptoHandle`](Self::DatawriterCryptoHandle) of the sending `DataWriter`.
    /// * `plain_rtps_submessage` - Output buffer for plain RTPS submessage.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case decoding fails.
    fn decode_datawriter_submessage(
        &mut self,
        encoded_rtps_submessage: &[u8],
        receiving_datareader_crypto: &Self::DatareaderCryptoHandle,
        sending_datawriter_crypto: &Self::DatawriterCryptoHandle,
        plain_rtps_submessage: &mut [u8],
    ) -> Result<usize, SecurityException>;

    /// Decodes a DataReader RTPS submessage.
    ///
    /// # Arguments
    ///
    /// * `encoded_rtps_submessage` - The input containing the encoded RTPS submessages.
    /// * `receiving_datawriter_crypto` - The [`DatawriterCryptoHandle`](Self::DatawriterCryptoHandle) of the receiving `DataWriter`.
    /// * `sending_datareader_crypto` - The [`DatareaderCryptoHandle`](Self::DatareaderCryptoHandle) of the sending `DataReader`.
    /// * `plain_rtps_submessage` - Output buffer for plain RTPS submessage.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case decoding fails.
    fn decode_datareader_submessage(
        &mut self,
        encoded_rtps_submessage: &[u8],
        receiving_datawriter_crypto: &Self::DatawriterCryptoHandle,
        sending_datareader_crypto: &Self::DatareaderCryptoHandle,
        plain_rtps_submessage: &mut [u8],
    ) -> Result<usize, SecurityException>;

    /// Decodes a `CryptoContent` submessage element into a `SerializedPayload`.
    ///
    /// # Arguments
    ///
    /// * `encoded_buffer` - The input containing the `CryptoContent` RTPS submessage element.
    /// * `inline_qos` - Inline QoS parameters.
    /// * `receiving_reader_crypto` - The [`DatareaderCryptoHandle`](Self::DatareaderCryptoHandle) of the receiving `DataReader`.
    /// * `sending_datawriter_crypto` - The [`DatawriterCryptoHandle`](Self::DatawriterCryptoHandle) of the sending `DataWriter`.
    /// * `plain_buffer` - Output buffer for `SerializedPayload`.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case decoding fails.
    fn decode_serialized_payload(
        &mut self,
        encoded_buffer: &[u8],
        inline_qos: &[u8],
        receiving_reader_crypto: &Self::DatareaderCryptoHandle,
        sending_datawriter_crypto: &Self::DatawriterCryptoHandle,
        plain_buffer: &mut [u8],
    ) -> Result<usize, SecurityException>;
}

impl CryptoTransform for () {
    type DatawriterCryptoHandle = ();
    type DatareaderCryptoHandle = ();
    type ParticipantCryptoHandle = ();

    fn encode_serialized_payload(
        &mut self,
        _plain_buffer: &[u8],
        _sending_datawriter_crypto: &Self::DatawriterCryptoHandle,
        _encoded_buffer: &mut [u8],
        _extra_inline_qos: &mut [u8],
    ) -> Result<EncodeSerializedPayloadOut, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn encode_datawriter_submessage(
        &mut self,
        _plain_rtps_submessage: &[u8],
        _sending_datawriter_crypto: &Self::DatawriterCryptoHandle,
        _receiving_datareader_crypto_list: &[Self::DatareaderCryptoHandle],
        _receiving_datareader_crypto_list_index: usize,
        _encoded_rtps_submessage: &mut [u8],
    ) -> Result<EncodeDatawriterSubmessageOut, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn encode_datareader_submessage(
        &mut self,
        _plain_rtps_submessage: &[u8],
        _sending_datareader_crypto: &Self::DatareaderCryptoHandle,
        _receiving_datawriter_crypto_list: &[Self::DatawriterCryptoHandle],
        _encoded_rtps_submessage: &mut [u8],
    ) -> Result<usize, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn encode_rtps_message(
        &mut self,
        _plain_rtps_message: &[u8],
        _sending_participant_crypto: &Self::ParticipantCryptoHandle,
        _receiving_participant_crypto_list: &[Self::ParticipantCryptoHandle],
        _receiving_participant_crypto_list_index: usize,
        _transform_with_psk: bool,
        _encoded_rtps_message: &mut [u8],
    ) -> Result<Option<EncodeRtpsMessageOut>, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn decode_rtps_message(
        &mut self,
        _encoded_rtps_message: &[u8],
        _receiving_participant_crypto: &Self::ParticipantCryptoHandle,
        _sending_participant_crypto: &Self::ParticipantCryptoHandle,
        _plain_rtps_message: &mut [u8],
    ) -> Result<usize, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn preprocess_secure_submsg(
        &mut self,
        _encoded_rtps_submessage: &[u8],
        _receiving_participant_crypto: &Self::ParticipantCryptoHandle,
        _sending_participant_crypto: &Self::ParticipantCryptoHandle,
    ) -> Result<
        PreprocessSecureSubmsgOut<Self::DatawriterCryptoHandle, Self::DatareaderCryptoHandle>,
        SecurityException,
    > {
        unreachable!("Placeholder should never be called")
    }

    fn decode_datawriter_submessage(
        &mut self,
        _encoded_rtps_submessage: &[u8],
        _receiving_datareader_crypto: &Self::DatareaderCryptoHandle,
        _sending_datawriter_crypto: &Self::DatawriterCryptoHandle,
        _plain_rtps_submessage: &mut [u8],
    ) -> Result<usize, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn decode_datareader_submessage(
        &mut self,
        _encoded_rtps_submessage: &[u8],
        _receiving_datawriter_crypto: &Self::DatawriterCryptoHandle,
        _sending_datareader_crypto: &Self::DatareaderCryptoHandle,
        _plain_rtps_submessage: &mut [u8],
    ) -> Result<usize, SecurityException> {
        unreachable!("Placeholder should never be called")
    }

    fn decode_serialized_payload(
        &mut self,
        _encoded_buffer: &[u8],
        _inline_qos: &[u8],
        _receiving_reader_crypto: &Self::DatareaderCryptoHandle,
        _sending_datawriter_crypto: &Self::DatawriterCryptoHandle,
        _plain_buffer: &mut [u8],
    ) -> Result<usize, SecurityException> {
        unreachable!("Placeholder should never be called")
    }
}
