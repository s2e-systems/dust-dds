#[derive(Debug)]
pub struct XtypesError(std::io::Error);

impl XtypesError {
    pub fn new(kind: std::io::ErrorKind, error: String) -> Self {
        Self(std::io::Error::new(kind, error))
    }
    pub fn invalid_data(error: String) -> Self {
        Self(std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    }
}

impl ToString for XtypesError {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<std::io::Error> for XtypesError {
    fn from(e: std::io::Error) -> Self {
        XtypesError(e)
    }
}

impl From<std::num::ParseIntError> for XtypesError {
    fn from(e: std::num::ParseIntError) -> Self {
        XtypesError::invalid_data(e.to_string())
    }
}

impl From<std::num::ParseFloatError> for XtypesError {
    fn from(e: std::num::ParseFloatError) -> Self {
        XtypesError::invalid_data(e.to_string())
    }
}

impl From<std::str::ParseBoolError> for XtypesError {
    fn from(e: std::str::ParseBoolError) -> Self {
        XtypesError::invalid_data(e.to_string())
    }
}

impl From<std::char::ParseCharError> for XtypesError {
    fn from(e: std::char::ParseCharError) -> Self {
        XtypesError::invalid_data(e.to_string())
    }
}
