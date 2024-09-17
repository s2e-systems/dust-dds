use dust_dds_derive::TypeSupport;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Bytes<'a>(pub &'a [u8]);

#[cfg(feature = "std")]
#[derive(Debug, PartialEq, Eq, Clone, TypeSupport)]
pub struct ByteBuf(pub Vec<u8>);
