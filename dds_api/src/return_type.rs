pub type DDSResult<T> = Result<T, DDSError>;

#[derive(Debug, PartialEq)]
pub enum DDSError {
    Error,
    Unsupported,
    BadParameter,
    PreconditionNotMet(&'static str),
    OutOfResources,
    NotEnabled,
    ImmutablePolicy,
    InconsistentPolicy,
    AlreadyDeleted,
    Timeout,
    NoData,
    IllegalOperation,
}
