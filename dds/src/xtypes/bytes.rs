use dust_dds_derive::TypeSupport;

use alloc::vec::Vec;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Bytes<'a>(pub &'a [u8]);

#[derive(Debug, PartialEq, Eq, Clone, TypeSupport)]
pub struct ByteBuf(pub Vec<u8>);
