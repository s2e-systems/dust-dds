use super::Submessage;

pub trait Pad: Submessage {
    fn new() -> Self;
}
