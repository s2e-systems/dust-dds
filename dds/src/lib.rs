#![doc = include_str!("../README.md")]
#![doc = include_str!("../schema/schema.md")]
#[forbid(unsafe_code)]
/// Module containing the traits and classes needed to represent types in the formats defined in the RTPS standard
pub mod cdr;
mod dds;
pub use dds::*;
mod implementation;

// To enable using our own derive macros
extern crate self as dust_dds;
