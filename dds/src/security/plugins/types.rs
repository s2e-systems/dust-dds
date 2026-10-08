use alloc::string::String;

/// SecurityException data type used to hold error information as defined in Section 9.2.1 of the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct SecurityException {
    /// Error message.
    pub message: String,
    /// Error code.
    pub code: i32,
    /// Minor error code.
    pub minor_code: i32,
}

pub struct DdsSecurityPlugins<Auth, Access, Crypto> {
    pub authentication_plugin: Auth,
    pub access_control_plugin: Access,
    pub cryptographic_plugin: Crypto,
}

impl DdsSecurityPlugins<(), (), ()> {
    /// Convenience constructor when security is disabled.
    pub fn disabled() -> Option<Self> {
        None
    }
}
