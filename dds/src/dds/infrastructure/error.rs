/// Result type returned by the different operations of the service
pub type DdsResult<T> = Result<T, DdsError>;

/// Errors that can be return by the different operations of the service
#[derive(Debug, PartialEq, Eq)]
pub enum DdsError {
    /// Generic, unspecified error.
    Error(String),
    /// Unsupported operation.
    Unsupported,
    /// Illegal parameter value.
    BadParameter,
    /// A pre-condition for the operation was not met.
    PreconditionNotMet(String),
    /// Service ran out of the resources needed to complete the operation.
    OutOfResources,
    /// Operation invoked on an Entity that is not yet enabled.
    NotEnabled,
    /// Application attempted to modify an immutable QosPolicy.
    ImmutablePolicy,
    /// Application specified a set of policies that are not consistent with each other.
    InconsistentPolicy,
    /// The object target of this operation has already been deleted.
    AlreadyDeleted,
    /// The operation timed out.
    Timeout,
    /// Indicates a transient situation where the operation did not
    /// return any data but there is no inherent error.
    NoData,
    /// An operation was invoked on an inappropriate object or
    /// at an inappropriate time (as determined by policies set by the
    /// specification or the Service implementation). There is no
    /// precondition that could be changed to make the operation
    /// succeed.
    IllegalOperation,
}

/// Return code representing the different errors
pub type ReturnCode = i32;

// const RETCODE_OK: ReturnCode = 0;
const RETCODE_ERROR: ReturnCode = 1;
const RETCODE_UNSUPPORTED: ReturnCode = 2;
const RETCODE_BAD_PARAMETER: ReturnCode = 3;
const RETCODE_PRECONDITION_NOT_MET: ReturnCode = 4;
const RETCODE_OUT_OF_RESOURCES: ReturnCode = 5;
const RETCODE_NOT_ENABLED: ReturnCode = 6;
const RETCODE_IMMUTABLE_POLICY: ReturnCode = 7;
const RETCODE_INCONSISTENT_POLICY: ReturnCode = 8;
const RETCODE_ALREADY_DELETED: ReturnCode = 9;
const RETCODE_TIMEOUT: ReturnCode = 10;
const RETCODE_NO_DATA: ReturnCode = 11;
const RETCODE_ILLEGAL_OPERATION: ReturnCode = 12;

impl From<DdsError> for ReturnCode {
    fn from(e: DdsError) -> Self {
        match e {
            DdsError::Error(_) => RETCODE_ERROR,
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
