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

pub struct DdsSecurityPlugins<Auth, Access> {
    pub authentication_plugin: Option<Auth>,
    pub access_control_plugin: Option<Access>,
}

impl DdsSecurityPlugins<(), ()> {
    /// Convenience constructor when security is disabled.
    pub fn disabled() -> Self {
        Self {
            authentication_plugin: None,
            access_control_plugin: None,
        }
    }
}
