pub mod interoperability{pub mod test{#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
pub struct Inner {pub a:u8,pub b:u32,pub c:u16,}
#[derive(Debug, dust_dds::infrastructure::type_support::DdsType)]
pub struct Nested {pub inner:Inner,pub level:i64,pub other:i32,pub value_list:Vec<i64>,pub last:i32,}
}}