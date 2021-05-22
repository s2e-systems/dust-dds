#![no_std]

pub mod behavior;
pub mod messages;
pub mod structure;
// pub mod discovery;

pub trait PIM: structure::Types + behavior::Types + messages::Types + Sized + 'static {
    type DataSubmesage: messages::submessages::Data<Self>;
    type GapSubmessage: messages::submessages::Gap<Self>;
}

#[cfg(test)]
#[macro_use]
extern crate std;
#[cfg(test)]
#[macro_use]
extern crate alloc;
