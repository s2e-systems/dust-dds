use alloc::string::String;

use super::{access_control::AccessControl, authentication::Authentication};

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

pub struct DdsSecurityPlugins<Auth: Authentication, Access: AccessControl> {
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

impl<Auth: Authentication, Access: AccessControl> DdsSecurityPlugins<Auth, Access> {
    pub fn new(authentication: Auth, access_control: Access) -> Self {
        Self {
            authentication_plugin: Some(authentication),
            access_control_plugin: Some(access_control),
        }
    }
}
