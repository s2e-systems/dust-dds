#[derive(Debug, PartialEq, Eq)]
pub struct Bytes<'a>(pub &'a [u8]);

#[derive(Debug, PartialEq, Eq)]
pub struct ByteBuf(pub Vec<u8>);
