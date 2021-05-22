use crate::{
    messages::{self, Submessage},
    structure,
};

pub trait Pad<PSM: structure::Types + messages::Types>: Submessage<PSM> {}
