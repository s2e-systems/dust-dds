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

/// Opaque handle representing internal authentication state as defined in Section 9.3.2.3 of the DDS Security specification.
pub type IdentityHandle = usize;

/// Opaque handle representing internal permissions state as defined in Section 9.4.2.3 of the DDS Security specification.
pub type PermissionsHandle = usize;
