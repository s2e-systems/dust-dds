#![doc = include_str!("../README.md")]
#![doc = include_str!("../schema/schema.md")]
#[forbid(unsafe_code)]
mod dds;
pub use dds::*;
pub use topic_definition::type_support::DdsType;
mod implementation;
