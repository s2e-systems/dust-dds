use self::overall_structure::SubmessageHeaderRead;

pub mod overall_structure;
pub mod submessage_elements;
pub mod submessages;
pub mod types;

pub trait FromBytes<'a> {
    fn from_bytes<E: byteorder::ByteOrder>(v: &'a [u8]) -> Self;
}

trait SubmessageHeader {
    fn submessage_header(&self) -> SubmessageHeaderRead;
}

trait RtpsMap<'a>: SubmessageHeader {
    fn map<T: FromBytes<'a>>(&self, data: &'a [u8]) -> T {
        if self.submessage_header().endianness_flag() {
            T::from_bytes::<byteorder::LittleEndian>(data)
        } else {
            T::from_bytes::<byteorder::BigEndian>(data)
        }
    }
}

impl<'a, T: SubmessageHeader> RtpsMap<'a> for T {}
