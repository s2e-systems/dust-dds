#![no_std]

pub mod behavior;
pub mod messages;
pub mod structure;
// pub mod discovery;

#[cfg(test)]
#[macro_use]
extern crate std;
#[cfg(test)]
#[macro_use]
extern crate alloc;