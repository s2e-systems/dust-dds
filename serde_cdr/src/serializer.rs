use crate::{error::Result, unimplemented_compound::UnimplementedCompound};
use byteorder::{LittleEndian, WriteBytesExt};
use serde::{
    ser::{SerializeSeq, SerializeStruct, SerializeTuple},
    Serialize, Serializer,
};
use std::io::Write;

pub fn to_bytes<T: ?Sized>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut serializer = RtpsMessageSerializer { writer: Vec::new() };
    value.serialize(&mut serializer)?;
    Ok(serializer.writer)
}

pub struct RtpsMessageSerializer<W> {
    pub writer: W,
}

pub struct SerializeCompound<'a, W: 'a> {
    ser: &'a mut RtpsMessageSerializer<W>,
}

impl<'a, W: Write> SerializeStruct for SerializeCompound<'a, W> {
    type Ok = ();
    type Error = crate::error::Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<()> {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeSeq for SerializeCompound<'a, W> {
    type Ok = ();
    type Error = crate::error::Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeTuple for SerializeCompound<'a, W> {
    type Ok = ();
    type Error = crate::error::Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<Self::Ok> {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: Write> Serializer for &'a mut RtpsMessageSerializer<W> {
    type Ok = ();
    type Error = crate::error::Error;

    type SerializeSeq = SerializeCompound<'a, W>;
    type SerializeTuple = SerializeCompound<'a, W>;
    type SerializeTupleStruct = UnimplementedCompound;
    type SerializeTupleVariant = UnimplementedCompound;
    type SerializeMap = UnimplementedCompound;
    type SerializeStruct = SerializeCompound<'a, W>;
    type SerializeStructVariant = UnimplementedCompound;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        Ok(self.writer.write_u8(v as u8)?)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        Ok(self.writer.write_i8(v)?)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        Ok(self.writer.write_i16::<LittleEndian>(v)?)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        Ok(self.writer.write_i32::<LittleEndian>(v)?)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        Ok(self.writer.write_i64::<LittleEndian>(v)?)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        Ok(self.writer.write_u8(v)?)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        Ok(self.writer.write_u16::<LittleEndian>(v)?)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        Ok(self.writer.write_u32::<LittleEndian>(v)?)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        Ok(self.writer.write_u64::<LittleEndian>(v)?)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        Ok(self.writer.write_f32::<LittleEndian>(v)?)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        Ok(self.writer.write_f64::<LittleEndian>(v)?)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.writer.write(v.to_string().as_bytes())?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.writer.write(v.as_bytes())?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.writer.write(v)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        match len {
            Some(len) => {
                if len > std::u32::MAX as usize {
                    return Err(Self::Error::NumberOutOfRange);
                }
                self.serialize_u32(len as u32)?
            }
            None => (),
        };
        Ok(SerializeCompound { ser: self })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(SerializeCompound { ser: self })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        todo!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        todo!()
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeCompound { ser: self })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Default for RtpsMessageSerializer<Vec<u8>> {
        fn default() -> Self {
            Self {
                writer: Vec::<u8>::new(),
            }
        }
    }

    fn serialize<T: serde::Serialize>(value: T) -> Vec<u8> {
        let mut serializer = RtpsMessageSerializer {
            writer: Vec::<u8>::new(),
        };
        value.serialize(&mut serializer).unwrap();
        serializer.writer
    }

    #[test]
    fn serialize_bool() {
        let mut serializer = RtpsMessageSerializer::default();

        serializer.serialize_bool(true).unwrap();
        assert_eq!(serializer.writer, vec![1]);

        let mut serializer = RtpsMessageSerializer::default();
        serializer.serialize_bool(false).unwrap();
        assert_eq!(serializer.writer, vec![0]);
    }

    #[test]
    fn serialize_u8() {
        let mut serializer = RtpsMessageSerializer::default();
        serializer.serialize_u8(4).unwrap();
        assert_eq!(serializer.writer, vec![4]);
    }

    #[test]
    fn serialize_multiple_u8() {
        let mut serializer = RtpsMessageSerializer::default();
        serializer.serialize_u8(4).unwrap();
        assert_eq!(serializer.writer, vec![4]);
        serializer.serialize_u8(5).unwrap();
        assert_eq!(serializer.writer, vec![4, 5]);
    }
    #[derive(Serialize)]
    struct SubmessageHeader {
        submessage_id: u8,
        flags: u8,
        octets_to_next_header: u16,
    }
    #[derive(Serialize)]
    struct Timestamp {
        seconds: i32,
        fraction: u32,
    }
    #[derive(Serialize)]
    struct InfoTimestampSubmessage {
        header: SubmessageHeader,
        timestamp: Timestamp,
    }

    #[test]
    fn serialize_submessage() {
        let submessage = InfoTimestampSubmessage {
            header: SubmessageHeader {
                submessage_id: 0x09,
                flags: 0b_0000_0001,
                octets_to_next_header: 8,
            },
            timestamp: Timestamp {
                seconds: 4,
                fraction: 2,
            },
        };

        let mut serializer = RtpsMessageSerializer::default();
        submessage.serialize(&mut serializer).unwrap();
        #[rustfmt::skip]
        assert_eq!(serializer.writer, vec![
            0x09, 0b_0000_0001, 8, 0, // Submessage header
            4, 0, 0, 0,               // Timestamp: seconds
            2, 0, 0, 0,               // Timestamp: fraction
        ]);
    }

    #[derive(Serialize)]
    struct NewType(u8);

    #[test]
    fn serialize_newtype() {
        let data = NewType(1);
        let mut serializer = RtpsMessageSerializer::default();
        data.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.writer, vec![1]);
    }

    #[derive(Serialize)]
    struct Vector(Vec<u8>);

    #[test]
    fn serialize_vector() {
        let data = Vector(vec![1, 2]);
        let mut serializer = RtpsMessageSerializer::default();
        data.serialize(&mut serializer).unwrap();
        #[rustfmt::skip]
        assert_eq!(serializer.writer, vec![
            2, 0, 0, 0, // Length
            1, 2 // Data
        ]);
    }

    #[derive(Serialize)]
    struct TupleArray([u8; 2]);
    #[derive(Serialize)]
    struct TupleTuple((u8, u8));

    #[test]
    fn serialize_tuple_array() {
        let array = TupleArray([1, 2]);
        let mut serializer = RtpsMessageSerializer::default();
        array.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.writer, vec![1, 2]);
    }

    #[test]
    fn serialize_tuple_tuple() {
        let tuple = TupleTuple((1, 2));
        let mut serializer = RtpsMessageSerializer::default();
        tuple.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.writer, vec![1, 2]);
    }

    #[derive(PartialEq, Debug, Serialize)]
    struct ReferenceStruct<'a> {
        length: u8,
        #[serde(with = "serde_bytes")]
        data: &'a [u8],
    }

    #[test]
    fn serialize_reference_struct() {
        let value = ReferenceStruct {
            length: 2,
            data: &[3, 4],
        };

        assert_eq!(serialize(value), vec![2, 3, 4]);
    }

    #[test]
    fn serialize_u8_json() {
        let expected = r#"1"#;
        let result = serde_json::ser::to_string(&1_u8).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn serialize_timestamp_json() {
        let expected = r#"{"seconds":1,"fraction":2}"#;
        let result = serde_json::ser::to_string(&Timestamp{ seconds:1 , fraction:2 }).unwrap();
        assert_eq!(result, expected);
    }
}
