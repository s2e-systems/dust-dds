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

/// BuiltinEndpointSetExt type as defined in Section 7.5.5 of the DDS Security specification (Table 12).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, TypeSupport)]
#[dust_dds(extensibility = "final", nested)]
pub struct BuiltinEndpointSetExt(pub u32);

impl BuiltinEndpointSetExt {
    #[allow(dead_code)]
    pub const TYPE_LOOKUP_SERVICE_REQUEST_SECURE_WRITER: u32 = 1 << 0;
    #[allow(dead_code)]
    pub const TYPE_LOOKUP_SERVICE_REQUEST_SECURE_READER: u32 = 1 << 1;
    #[allow(dead_code)]
    pub const TYPE_LOOKUP_SERVICE_REPLY_SECURE_WRITER: u32 = 1 << 2;
    #[allow(dead_code)]
    pub const TYPE_LOOKUP_SERVICE_REPLY_SECURE_READER: u32 = 1 << 3;

    #[allow(dead_code)]
    pub const BUILTIN_ENDPOINT_TYPE_LOOKUP_SERVICE_REQUEST_SECURE_WRITER: u32 =
        Self::TYPE_LOOKUP_SERVICE_REQUEST_SECURE_WRITER;
    #[allow(dead_code)]
    pub const BUILTIN_ENDPOINT_TYPE_LOOKUP_SERVICE_REQUEST_SECURE_READER: u32 =
        Self::TYPE_LOOKUP_SERVICE_REQUEST_SECURE_READER;
    #[allow(dead_code)]
    pub const BUILTIN_ENDPOINT_TYPE_LOOKUP_SERVICE_REPLY_SECURE_WRITER: u32 =
        Self::TYPE_LOOKUP_SERVICE_REPLY_SECURE_WRITER;
    #[allow(dead_code)]
    pub const BUILTIN_ENDPOINT_TYPE_LOOKUP_SERVICE_REPLY_SECURE_READER: u32 =
        Self::TYPE_LOOKUP_SERVICE_REPLY_SECURE_READER;

    #[allow(dead_code)]
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    #[allow(dead_code)]
    pub fn has(&self, endpoint: u32) -> bool {
        (self.0 & endpoint) == endpoint
    }
}

/// CryptoAlgorithmSet type as defined in Section 7.3.10 of the DDS Security specification.
pub type CryptoAlgorithmSet = u32;

/// CryptoAlgorithmBit type as defined in Section 7.3.10 of the DDS Security specification.
pub type CryptoAlgorithmBit = u32;

/// Bitmask value representing all cryptographic algorithms set.
pub const CRYPTO_ALGORITHM_SET_ALL: CryptoAlgorithmSet = 0xffff_ffff;

/// Bitmask value representing an empty set of cryptographic algorithms.
pub const CRYPTO_ALGORITHM_SET_EMPTY: CryptoAlgorithmSet = 0x0000_0000;

/// Bit indicating compatibility mode in CryptoAlgorithmRequirements.
pub const CRYPTO_ALGORITHM_COMPATIBILITY_MODE: CryptoAlgorithmBit = 0x8000_0000;

/// Predefined CryptoAlgorithmBit value for DHE+MODP-2048-256.
pub const CBIT_DHE_MODP_2048_256: CryptoAlgorithmBit = 1 << 0;
/// Predefined CryptoAlgorithmBit value for ECDHE-CEUM+P256.
pub const CBIT_ECDHE_CEUM_P256: CryptoAlgorithmBit = 1 << 1;
/// Predefined CryptoAlgorithmBit value for ECDHE-CEUM+P384.
pub const CBIT_ECDHE_CEUM_P384: CryptoAlgorithmBit = 1 << 2;

/// Predefined CryptoAlgorithmBit value for AES128+GMAC.
pub const CBIT_AES128_GMAC: CryptoAlgorithmBit = 1 << 0;
/// Predefined CryptoAlgorithmBit value for AES128+GCM.
pub const CBIT_AES128_GCM: CryptoAlgorithmBit = 1 << 0;
/// Predefined CryptoAlgorithmBit value for AES256+GMAC.
pub const CBIT_AES256_GMAC: CryptoAlgorithmBit = 1 << 1;
/// Predefined CryptoAlgorithmBit value for AES256+GCM.
pub const CBIT_AES256_GCM: CryptoAlgorithmBit = 1 << 1;

/// CryptoAlgorithmRequirements type as defined in Section 7.3.10 of the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct CryptoAlgorithmRequirements {
    /// Bitmask of supported algorithms.
    pub supported_mask: CryptoAlgorithmSet,
    /// Bitmask of required algorithms.
    pub required_mask: CryptoAlgorithmSet,
}

impl CryptoAlgorithmRequirements {
    /// Checks whether two [`CryptoAlgorithmRequirements`] configurations are compatible.
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        let check_compatibility =
            |supported_mask: CryptoAlgorithmSet, required_mask: CryptoAlgorithmSet| {
                ((required_mask & supported_mask) == required_mask)
                    || (((required_mask & supported_mask) != 0)
                        && ((required_mask & CRYPTO_ALGORITHM_COMPATIBILITY_MODE) != 0))
            };

        check_compatibility(other.supported_mask, self.required_mask)
            && check_compatibility(self.supported_mask, other.required_mask)
    }
}

/// ParticipantSecurityDigitalSignatureAlgorithmInfo type as defined in Section 7.5.1.4 of the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, TypeSupport)]
#[dust_dds(extensibility = "appendable")]
pub struct ParticipantSecurityDigitalSignatureAlgorithmInfo {
    /// Trust chain algorithm requirements.
    pub trust_chain: CryptoAlgorithmRequirements,
    /// Message authentication algorithm requirements.
    pub message_auth: CryptoAlgorithmRequirements,
}

impl ParticipantSecurityDigitalSignatureAlgorithmInfo {
    /// Checks whether two [`ParticipantSecurityDigitalSignatureAlgorithmInfo`] configurations are compatible.
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.trust_chain.is_compatible_with(&other.trust_chain)
            && self.message_auth.is_compatible_with(&other.message_auth)
    }
}

/// ParticipantSecurityKeyEstablishmentAlgorithmInfo type as defined in Section 7.3.12 of the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Copy, TypeSupport)]
#[dust_dds(extensibility = "appendable")]
pub struct ParticipantSecurityKeyEstablishmentAlgorithmInfo {
    /// Shared secret algorithm requirements.
    pub shared_secret: CryptoAlgorithmRequirements,
}

impl Default for ParticipantSecurityKeyEstablishmentAlgorithmInfo {
    fn default() -> Self {
        Self {
            shared_secret: CryptoAlgorithmRequirements {
                supported_mask: CBIT_DHE_MODP_2048_256 | CBIT_ECDHE_CEUM_P256,
                required_mask: CBIT_ECDHE_CEUM_P256,
            },
        }
    }
}

impl ParticipantSecurityKeyEstablishmentAlgorithmInfo {
    /// Checks whether two [`ParticipantSecurityKeyEstablishmentAlgorithmInfo`] configurations are compatible.
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.shared_secret.is_compatible_with(&other.shared_secret)
    }
}

/// ParticipantSecuritySymmetricCipherAlgorithmInfo type as defined in Section 7.3.13 of the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Copy, TypeSupport)]
#[dust_dds(extensibility = "appendable")]
pub struct ParticipantSecuritySymmetricCipherAlgorithmInfo {
    /// Supported algorithms mask.
    pub supported_mask: CryptoAlgorithmSet,
    /// Required algorithms mask for builtin endpoints.
    pub builtin_endpoints_required_mask: CryptoAlgorithmSet,
    /// Required algorithms mask for builtin key exchange endpoints.
    pub builtin_kx_endpoints_required_mask: CryptoAlgorithmSet,
    /// Default required algorithms mask for user endpoints.
    pub user_endpoints_default_required_mask: CryptoAlgorithmSet,
}

impl Default for ParticipantSecuritySymmetricCipherAlgorithmInfo {
    fn default() -> Self {
        Self {
            supported_mask: CBIT_AES128_GCM | CBIT_AES256_GCM,
            builtin_endpoints_required_mask: CBIT_AES256_GCM,
            builtin_kx_endpoints_required_mask: CBIT_AES256_GCM,
            user_endpoints_default_required_mask: CBIT_AES256_GCM,
        }
    }
}

impl ParticipantSecuritySymmetricCipherAlgorithmInfo {
    /// Checks whether two [`ParticipantSecuritySymmetricCipherAlgorithmInfo`] configurations are compatible.
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        let check_compatibility =
            |supported_mask: CryptoAlgorithmSet, required_mask: CryptoAlgorithmSet| {
                ((required_mask & supported_mask) == required_mask)
                    || (((required_mask & supported_mask) != 0)
                        && ((required_mask & CRYPTO_ALGORITHM_COMPATIBILITY_MODE) != 0))
            };

        check_compatibility(other.supported_mask, self.builtin_endpoints_required_mask)
            && check_compatibility(
                other.supported_mask,
                self.builtin_kx_endpoints_required_mask,
            )
            && check_compatibility(self.supported_mask, other.builtin_endpoints_required_mask)
            && check_compatibility(
                self.supported_mask,
                other.builtin_kx_endpoints_required_mask,
            )
    }
}

/// ParticipantSecurityAlgorithmInfo type as defined in Section 7.3.14 of the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, TypeSupport)]
#[dust_dds(extensibility = "appendable")]
pub struct ParticipantSecurityAlgorithmInfo {
    /// Digital signature algorithm info.
    pub digital_signature: ParticipantSecurityDigitalSignatureAlgorithmInfo,
    /// Key establishment algorithm info.
    pub key_establishment: ParticipantSecurityKeyEstablishmentAlgorithmInfo,
    /// Symmetric cipher algorithm info.
    pub symmetric_cipher: ParticipantSecuritySymmetricCipherAlgorithmInfo,
}

/// ParticipantSecurityConfig type as defined in Section 7.3 of the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "appendable")]
pub struct ParticipantSecurityConfig {
    /// Indicates whether unauthenticated participants are allowed.
    pub allow_unauthenticated_participants: bool,
    /// Indicates whether access control is protected.
    pub is_access_protected: bool,
    /// Indicates whether RTPS AXK is protected.
    pub is_rtps_axk_protected: bool,
    /// Indicates whether RTPS PSK is protected.
    pub is_rtps_psk_protected: bool,
    /// Indicates whether discovery is protected.
    pub is_discovery_protected: bool,
    /// Indicates whether liveliness is protected.
    pub is_liveliness_protected: bool,
    /// Indicates whether key revision is enabled.
    pub is_key_revision_enabled: bool,
    /// Plugin participant attributes.
    pub plugin_participant_attributes: PluginParticipantSecurityAttributesMask,
    /// AC participant / endpoint properties.
    pub ac_endpoint_properties: PropertySeq,
    /// Cryptographic algorithms used and supported by the participant.
    pub algorithm_info: ParticipantSecurityAlgorithmInfo,
}

/// EndpointSecurityAttributesMask type as defined in Section 7.3.24 of the DDS Security specification.
pub type EndpointSecurityAttributesMask = u32;

/// PluginEndpointSecurityAttributesMask type as defined in Section 7.3.24 of the DDS Security specification.
pub type PluginEndpointSecurityAttributesMask = u32;

/// Flag indicating whether the mask is valid in EndpointSecurityProtectionInfo.
pub const ENDPOINT_SECURITY_ATTRIBUTES_FLAG_IS_VALID: u32 = 0x1 << 31;

/// Default value for [`EndpointSecurityProtectionInfo`].
pub const ENDPOINT_SECURITY_ATTRIBUTES_INFO_DEFAULT: EndpointSecurityProtectionInfo =
    EndpointSecurityProtectionInfo {
        endpoint_security_attributes: 0,
        plugin_endpoint_security_attributes: 0,
    };

/// EndpointSecurityProtectionInfo type as defined in Section 7.3.24 of the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, TypeSupport)]
#[dust_dds(extensibility = "appendable")]
pub struct EndpointSecurityProtectionInfo {
    /// Endpoint security attributes mask.
    pub endpoint_security_attributes: EndpointSecurityAttributesMask,
    /// Plugin endpoint security attributes mask.
    pub plugin_endpoint_security_attributes: PluginEndpointSecurityAttributesMask,
}

impl EndpointSecurityProtectionInfo {
    /// Checks whether two [`EndpointSecurityProtectionInfo`] configurations are compatible.
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        let mask_compatible = |mask1: u32, mask2: u32| {
            let valid1 = (mask1 & ENDPOINT_SECURITY_ATTRIBUTES_FLAG_IS_VALID) != 0;
            let valid2 = (mask2 & ENDPOINT_SECURITY_ATTRIBUTES_FLAG_IS_VALID) != 0;
            if valid1 && valid2 {
                mask1 == mask2
            } else {
                true
            }
        };

        mask_compatible(
            self.endpoint_security_attributes,
            other.endpoint_security_attributes,
        ) && mask_compatible(
            self.plugin_endpoint_security_attributes,
            other.plugin_endpoint_security_attributes,
        )
    }
}

/// EndpointSecuritySymmetricCipherAlgorithmInfo type as defined in Section 7.3.15 of the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, TypeSupport)]
#[dust_dds(extensibility = "appendable")]
pub struct EndpointSecuritySymmetricCipherAlgorithmInfo {
    /// Required algorithms mask.
    pub required_mask: CryptoAlgorithmSet,
    /// Supported algorithms mask (non-serialized).
    #[dust_dds(non_serialized)]
    pub supported_mask: CryptoAlgorithmSet,
}

impl EndpointSecuritySymmetricCipherAlgorithmInfo {
    /// Checks whether the [`EndpointSecuritySymmetricCipherAlgorithmInfo`] of two endpoints are compatible according to Section 7.3.15.1.
    pub fn is_compatible_with(
        &self,
        participant_supported_mask: CryptoAlgorithmSet,
        other: &Self,
        other_participant_supported_mask: CryptoAlgorithmSet,
    ) -> bool {
        let check_compatibility =
            |supported_mask: CryptoAlgorithmSet, required_mask: CryptoAlgorithmSet| {
                ((required_mask & supported_mask) == required_mask)
                    || (((required_mask & supported_mask) != 0)
                        && ((required_mask & CRYPTO_ALGORITHM_COMPATIBILITY_MODE) != 0))
            };

        check_compatibility(other_participant_supported_mask, self.required_mask)
            && check_compatibility(participant_supported_mask, other.required_mask)
    }
}

/// TopicSecurityConfig type as defined in the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "appendable")]
pub struct TopicSecurityConfig {
    /// Indicates whether read operations are protected.
    pub is_read_protected: bool,
    /// Indicates whether write operations are protected.
    pub is_write_protected: bool,
    /// Indicates whether discovery is protected.
    pub is_discovery_protected: bool,
    /// Indicates whether liveliness is protected.
    pub is_liveliness_protected: bool,
    /// Plugin endpoint security attributes mask.
    pub plugin_endpoint_attributes: PluginEndpointSecurityAttributesMask,
    /// AC endpoint properties.
    pub ac_endpoint_properties: PropertySeq,
}

/// EndpointSecurityAlgorithmInfo type as defined in Section 7.3.15 of the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, TypeSupport)]
#[dust_dds(extensibility = "appendable")]
pub struct EndpointSecurityAlgorithmInfo {
    /// Symmetric cipher algorithm info.
    pub symmetric_cipher: EndpointSecuritySymmetricCipherAlgorithmInfo,
}

/// EndpointSecurityConfig type as defined in Section 9.4.2.7 of the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "appendable", base_type = TopicSecurityConfig)]
pub struct EndpointSecurityConfig {
    /// Parent [`TopicSecurityConfig`].
    pub parent: TopicSecurityConfig,
    /// Indicates whether submessages are protected.
    pub is_submessage_protected: bool,
    /// Indicates whether payload is protected.
    pub is_payload_protected: bool,
    /// Indicates whether key is protected.
    pub is_key_protected: bool,
    /// Plugin endpoint security attributes mask.
    pub plugin_endpoint_attributes: PluginEndpointSecurityAttributesMask,
    /// AC endpoint properties.
    pub ac_endpoint_properties: PropertySeq,
    /// Endpoint security algorithm info.
    pub algorithm_info: EndpointSecurityAlgorithmInfo,
}

/// CryptoTransformKeyRevision type as defined in Section 7.3.17 of the DDS Security specification.
pub type CryptoTransformKeyRevision = [u8; 3];

/// CryptoTransformKeyRevisionIntHolder type as defined in Section 7.3.17 of the DDS Security specification.
pub type CryptoTransformKeyRevisionIntHolder = i32;

/// Constant representing no key revision.
pub const CRYPTO_TRANSFORM_KEY_REVISION_NONE: CryptoTransformKeyRevision = [0x00, 0x00, 0x00];
