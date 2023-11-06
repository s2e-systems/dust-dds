/// Type for the instance handle representing an Entity
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    Hash,
    PartialOrd,
    Ord,
    derive_more::Constructor,
    derive_more::AsRef,
)]
pub struct InstanceHandle([u8; 16]);

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
