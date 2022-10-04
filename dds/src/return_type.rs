pub type DdsResult<T> = Result<T, DdsError>;

#[derive(Debug, PartialEq, Eq)]
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
