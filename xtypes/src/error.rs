#[derive(Debug, PartialEq)]
pub enum XcdrError {
    OutOfMemory,
    InvalidData,
    PidNotFound(u16),
}
