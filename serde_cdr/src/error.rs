#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    SequenceMustHaveLength,
}

impl From<std::io::Error> for Error {
    fn from(io_error: std::io::Error) -> Self {
        Self::IoError(io_error)
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(_msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        unimplemented!()
    }
}

impl serde::de::Error for Error {
    fn custom<T>(_msg:T)-> Self
        where T:std::fmt::Display
    {
        todo!()
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}
