/// Type for the instance handle representing an Entity
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct InstanceHandle([u8; 16]);

impl InstanceHandle {
    /// InstanceHandle constructor
    pub fn new(bytes: [u8; 16]) -> Self {
        InstanceHandle(bytes)
    }
}

impl AsRef<[u8; 16]> for InstanceHandle {
    fn as_ref(&self) -> &[u8; 16] {
        &self.0
    }
}

impl Default for InstanceHandle {
    fn default() -> Self {
        HANDLE_NIL
    }
}

/// Special constant value representing a 'nil' [`InstanceHandle`]
pub const HANDLE_NIL: InstanceHandle = InstanceHandle([0; 16]);

impl From<InstanceHandle> for [u8; 16] {
    fn from(x: InstanceHandle) -> Self {
        x.0
    }
}
