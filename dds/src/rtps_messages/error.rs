use crate::xtypes::error::XTypesError;

pub type RtpsMessageResult<T> = Result<T, RtpsMessageError>;

#[derive(Debug)]
pub enum RtpsMessageError {
    Io,
    InvalidData,
    NotEnoughData,
    UnknownMessage,
}

impl From<std::io::Error> for RtpsMessageError {
    fn from(_: std::io::Error) -> Self {
        RtpsMessageError::Io
    }
}

impl From<XTypesError> for RtpsMessageError {
    fn from(_: XTypesError) -> Self {
        RtpsMessageError::InvalidData
    }
}
