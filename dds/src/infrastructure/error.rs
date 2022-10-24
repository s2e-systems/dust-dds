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

// ----------------------------------------------------------------------
// Return codes
// ----------------------------------------------------------------------
pub type ReturnCode = i32;

pub const RETCODE_OK: ReturnCode = 0;
pub const RETCODE_ERROR: ReturnCode = 1;
pub const RETCODE_UNSUPPORTED: ReturnCode = 2;
pub const RETCODE_BAD_PARAMETER: ReturnCode = 3;
pub const RETCODE_PRECONDITION_NOT_MET: ReturnCode = 4;
pub const RETCODE_OUT_OF_RESOURCES: ReturnCode = 5;
pub const RETCODE_NOT_ENABLED: ReturnCode = 6;
pub const RETCODE_IMMUTABLE_POLICY: ReturnCode = 7;
pub const RETCODE_INCONSISTENT_POLICY: ReturnCode = 8;
pub const RETCODE_ALREADY_DELETED: ReturnCode = 9;
pub const RETCODE_TIMEOUT: ReturnCode = 10;
pub const RETCODE_NO_DATA: ReturnCode = 11;
pub const RETCODE_ILLEGAL_OPERATION: ReturnCode = 12;

impl From<DdsError> for ReturnCode {
    fn from(e: DdsError) -> Self {
        match e {
            DdsError::Error => RETCODE_ERROR,
            DdsError::Unsupported => RETCODE_UNSUPPORTED,
            DdsError::BadParameter => RETCODE_BAD_PARAMETER,
            DdsError::PreconditionNotMet(_) => RETCODE_PRECONDITION_NOT_MET,
            DdsError::OutOfResources => RETCODE_OUT_OF_RESOURCES,
            DdsError::NotEnabled => RETCODE_NOT_ENABLED,
            DdsError::ImmutablePolicy => RETCODE_IMMUTABLE_POLICY,
            DdsError::InconsistentPolicy => RETCODE_INCONSISTENT_POLICY,
            DdsError::AlreadyDeleted => RETCODE_ALREADY_DELETED,
            DdsError::Timeout => RETCODE_TIMEOUT,
            DdsError::NoData => RETCODE_NO_DATA,
            DdsError::IllegalOperation => RETCODE_ILLEGAL_OPERATION,
        }
    }
}
