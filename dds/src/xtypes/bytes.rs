#[derive(Debug, PartialEq, Eq)]
pub struct Bytes<'a>(pub &'a [u8]);

#[cfg(feature = "std")]
#[derive(Debug, PartialEq, Eq)]
pub struct ByteBuf(pub Vec<u8>);
