use alloc::{string::String, vec::Vec};
use dust_dds_derive::TypeSupport;

use crate::rtps::types::PropertySeq;

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
    pub properties: PropertySeq,
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

/// ParticipantSecurityAttributesMask type as defined in Section 7.3 of the DDS Security specification.
pub type ParticipantSecurityAttributesMask = u32;

/// PluginParticipantSecurityAttributesMask type as defined in Section 7.3 of the DDS Security specification.
pub type PluginParticipantSecurityAttributesMask = u32;

/// ParticipantSecurityAttributesMaskExt type as defined in Section 7.3 of the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct ParticipantSecurityAttributesMaskExt {
    /// Indicates whether the corresponding bit in the value mask is set.
    pub is_set: u16,
    /// Value mask.
    pub value: u16,
}

/// ParticipantSecurityProtectionInfo type as defined in Section 7.3 of the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, TypeSupport)]
#[dust_dds(extensibility = "appendable")]
pub struct ParticipantSecurityProtectionInfo {
    /// Participant security attributes mask.
    pub participant_security_attributes: ParticipantSecurityAttributesMask,
    /// Plugin participant security attributes mask.
    pub plugin_participant_security_attributes: PluginParticipantSecurityAttributesMask,
    /// Participant security optional attributes.
    pub participant_security_optional_attributes: ParticipantSecurityAttributesMaskExt,
}

/// Flag indicating whether the mask is valid in [`ParticipantSecurityAttributesMask`] and [`PluginParticipantSecurityAttributesMask`].
pub const PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_VALID: u32 = 0x1 << 31;

/// Default value for [`ParticipantSecurityProtectionInfo`].
pub const PARTICIPANT_SECURITY_ATTRIBUTES_INFO_DEFAULT: ParticipantSecurityProtectionInfo =
    ParticipantSecurityProtectionInfo {
        participant_security_attributes: 0,
        plugin_participant_security_attributes: 0,
        participant_security_optional_attributes: ParticipantSecurityAttributesMaskExt {
            is_set: 0,
            value: 0,
        },
    };

impl ParticipantSecurityProtectionInfo {
    /// Checks whether two [`ParticipantSecurityProtectionInfo`] configurations are compatible.
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        let mask_compatible = |mask1: u32, mask2: u32| {
            let valid1 = (mask1 & PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_VALID) != 0;
            let valid2 = (mask2 & PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_VALID) != 0;
            if valid1 && valid2 {
                mask1 == mask2
            } else {
                true
            }
        };

        mask_compatible(
            self.participant_security_attributes,
            other.participant_security_attributes,
        ) && mask_compatible(
            self.plugin_participant_security_attributes,
            other.plugin_participant_security_attributes,
        )
    }
}
