type HandleTypeNative = [u8; 16]; // Originally in the DDS idl i32
const HANDLE_NIL_NATIVE: HandleTypeNative = [0; 16];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct InstanceHandle(HandleTypeNative);

impl From<[u8; 16]> for InstanceHandle {
    fn from(x: [u8; 16]) -> Self {
        Self(x)
    }
}

impl From<InstanceHandle> for [u8; 16] {
    fn from(this: InstanceHandle) -> Self {
        this.0
    }
}

pub const HANDLE_NIL: InstanceHandle = InstanceHandle(HANDLE_NIL_NATIVE);