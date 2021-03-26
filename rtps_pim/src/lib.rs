// #![no_std]

// pub mod behavior;
pub mod messages;
pub mod structure;
// pub mod discovery;

pub trait RtpsPim : structure::Types + messages::Types {}