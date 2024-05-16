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

impl std::fmt::Display for RtpsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
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
