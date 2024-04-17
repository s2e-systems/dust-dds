pub type RtpsResult<T> = Result<T, RtpsError>;

#[derive(Debug)]
pub enum RtpsErrorKind {
    Io,
    InvalidData,
    NotEnoughData,
}

#[derive(Debug)]
pub struct RtpsError {
    kind: RtpsErrorKind,
    msg: String,
}

impl RtpsError {
    pub fn new(kind: RtpsErrorKind, msg: impl ToString) -> Self {
        Self {
            kind,
            msg: msg.to_string(),
        }
    }
}

impl ToString for RtpsError {
    fn to_string(&self) -> String {
        format!(
            "{}: {}",
            match self.kind {
                RtpsErrorKind::InvalidData => "Invalid data",
                RtpsErrorKind::NotEnoughData => "Not enough data",
                RtpsErrorKind::Io => "Io",
            },
            self.msg
        )
    }
}

impl From<std::io::Error> for RtpsError {
    fn from(e: std::io::Error) -> Self {
        RtpsError::new(RtpsErrorKind::Io, e)
    }
}
