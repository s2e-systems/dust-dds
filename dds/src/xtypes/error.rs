pub type XTypesResult<T> = Result<T, XTypesError>;

#[derive(Debug, PartialEq)]
pub enum XTypesError {
    OutOfMemory,
    InvalidData,
    InvalidType,
    PidNotFound(u16),
    InvalidId(u32),
    InvalidIndex(u32),
    NotEnoughData,
    NotSupported([u8; 2]),
    /// An operation was invoked on an inappropriate object or
    /// at an inappropriate time (as determined by policies set by the
    /// specification or the Service implementation). There is no
    /// precondition that could be changed to make the operation
    /// succeed.
    IllegalOperation,
}
