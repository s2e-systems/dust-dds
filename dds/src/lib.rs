#![doc = include_str!("../README.md")]
#![doc = include_str!("../schema/schema.md")]
#[forbid(unsafe_code)]
pub mod cdr;
mod dds;
pub use dds::*;
mod implementation;

// To enable using our own derive macros
extern crate self as dust_dds;
