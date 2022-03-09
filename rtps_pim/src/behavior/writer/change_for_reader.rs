use crate::behavior::types::ChangeForReaderStatusKind;

pub trait RtpsChangeForReaderAttributes {
    fn status(&mut self) -> &mut ChangeForReaderStatusKind;
    fn is_relevant(&self) -> bool;
}

pub trait RtpsChangeForReaderConstructor {
    fn new(status: ChangeForReaderStatusKind, is_relevant: bool) -> Self;
}
