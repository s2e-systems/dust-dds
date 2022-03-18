pub type DDSResult<T> = Result<T, DdsError>;

#[derive(Debug, PartialEq)]
pub enum DdsError {
    Error,
    Unsupported,
    BadParameter,
    PreconditionNotMet(String),
    OutOfResources,
    NotEnabled,
    ImmutablePolicy,
    InconsistentPolicy,
    AlreadyDeleted,
    Timeout,
    NoData,
    IllegalOperation,
}
