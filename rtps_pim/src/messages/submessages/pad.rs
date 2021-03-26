use crate::messages::Submessage;

pub trait Pad: Submessage {
    fn new() -> Self;
}
