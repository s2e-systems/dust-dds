use std::io::Write;

use crate::{
    rtps::error::{RtpsError, RtpsErrorKind},
    xtypes::{serialize::XTypesSerialize, xcdr_serializer::Xcdr1LeSerializer},
};

const PL_CDR_LE: [u8; 2] = [0x00, 0x03];
const REPRESENTATION_OPTIONS: [u8; 2] = [0x00, 0x00];
const PID_SENTINEL: i16 = 1;

#[derive(Default)]
pub struct ParameterListCdrSerializer {
    pub writer: Vec<u8>,
}

impl ParameterListCdrSerializer {
    pub fn write_header(&mut self) -> Result<(), RtpsError> {
        self.writer.write_all(&PL_CDR_LE)?;
        self.writer.write_all(&REPRESENTATION_OPTIONS)?;
        Ok(())
    }
    pub fn write_sentinel(&mut self) -> Result<(), RtpsError> {
        self.writer.write_all(&PID_SENTINEL.to_le_bytes())?;
        self.writer.write_all(&0_u16.to_le_bytes())?;
        Ok(())
    }

    pub fn write<T>(&mut self, id: i16, value: &T) -> Result<(), RtpsError>
    where
        T: XTypesSerialize,
    {
        let mut data = Vec::new();

        let mut data_serializer = Xcdr1LeSerializer::new(&mut data);
        value.serialize(&mut data_serializer)?;

        let length_without_padding = data.len();
        let padding_length = (4 - length_without_padding % 4) & 3;
        let length = length_without_padding + padding_length;

        if length > u16::MAX as usize {
            return Err(RtpsError::new(RtpsErrorKind::InvalidData, format!("Serialized parameter ID {} with serialized size {} exceeds maximum parameter size of {}", id, length, u16::MAX)));
        }

        self.writer.write_all(&id.to_le_bytes())?;
        self.writer.write_all(&(length as u16).to_le_bytes())?;

        self.writer.write_all(&data)?;

        match padding_length {
            1 => self.writer.write_all(&[0u8; 1])?,
            2 => self.writer.write_all(&[0u8; 2])?,
            3 => self.writer.write_all(&[0u8; 3])?,
            _ => self.writer.write_all(&[0u8; 0])?,
        }

        Ok(())
    }

    pub fn write_with_default<T>(
        &mut self,
        id: i16,
        value: &T,
        default: &T,
    ) -> Result<(), RtpsError>
    where
        T: XTypesSerialize + PartialEq,
    {
        if value != default {
            self.write(id, value)?;
        }
        Ok(())
    }

    pub fn write_collection<T>(&mut self, id: i16, value_list: &[T]) -> Result<(), RtpsError>
    where
        T: XTypesSerialize,
    {
        for value in value_list {
            self.write(id, value)?;
        }
        Ok(())
    }
}
