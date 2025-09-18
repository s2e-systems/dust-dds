pub type XTypesResult<T> = Result<T, XTypesError>;

#[derive(Debug, PartialEq)]
pub enum XTypesError {
    OutOfMemory,
    InvalidData,
    PidNotFound(u16),
    InvalidIndex,
}
