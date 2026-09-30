use alloc::{string::String, vec::Vec};
use dust_dds_derive::TypeSupport;

/// BinaryProperty type as defined in Section 7.3.3 of the DDS Security specification.
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct BinaryProperty {
    /// Name of the binary property.
    pub name: String,
    /// Value associated with that name.
    pub value: Vec<u8>,
    /// Indicates whether the binary property is intended for local use only or should be propagated by DDS discovery.
    #[dust_dds(non_serialized)]
    pub propagate: bool,
}

/// Sequence of [`BinaryProperty`].
pub type BinaryPropertySeq = Vec<BinaryProperty>;

/// DataHolder type as defined in Section 7.3.4 of the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct DataHolder {
    /// Class ID.
    pub class_id: String,
    /// Sequence of properties.
    pub properties: crate::rtps::types::PropertySeq,
    /// Sequence of binary properties.
    pub binary_properties: BinaryPropertySeq,
}

/// Sequence of [`DataHolder`].
pub type DataHolderSeq = Vec<DataHolder>;

/// Token type as defined in Section 7.3.6 of the DDS Security specification.
pub type Token = DataHolder;

/// MessageToken type as defined in Section 7.3.6 of the DDS Security specification.
pub type MessageToken = Token;

/// AuthRequestMessageToken type as defined in the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct AuthRequestMessageToken(pub MessageToken);

/// HandshakeMessageToken type as defined in the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct HandshakeMessageToken(pub MessageToken);

/// IdentityToken type as defined in the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct IdentityToken(pub Token);

/// IdentityStatusToken type as defined in the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct IdentityStatusToken(pub Token);

/// PermissionsToken type as defined in the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct PermissionsToken(pub Token);

/// AuthenticatedPeerCredentialToken type as defined in the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct AuthenticatedPeerCredentialToken(pub Token);

/// PermissionsCredentialToken type as defined in the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct PermissionsCredentialToken(pub Token);

/// CryptoToken type as defined in the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct CryptoToken(pub Token);

/// ParticipantCryptoToken type as defined in the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct ParticipantCryptoToken(pub Token);

/// DatawriterCryptoToken type as defined in the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct DatawriterCryptoToken(pub Token);

/// DatareaderCryptoToken type as defined in the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct DatareaderCryptoToken(pub Token);

/// Sequence of [`HandshakeMessageToken`].
pub type HandshakeMessageTokenSeq = Vec<HandshakeMessageToken>;

/// Sequence of [`CryptoToken`].
pub type CryptoTokenSeq = Vec<CryptoToken>;

/// Sequence of [`ParticipantCryptoToken`].
pub type ParticipantCryptoTokenSeq = CryptoTokenSeq;

/// Sequence of [`DatawriterCryptoToken`].
pub type DatawriterCryptoTokenSeq = CryptoTokenSeq;

/// Sequence of [`DatareaderCryptoToken`].
pub type DatareaderCryptoTokenSeq = CryptoTokenSeq;
