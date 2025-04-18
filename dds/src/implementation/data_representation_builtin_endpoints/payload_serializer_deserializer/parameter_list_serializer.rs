use crate::{
    rtps::error::{RtpsError, RtpsErrorKind},
    rtps_messages::types::ParameterId,
    xtypes::{serialize::XTypesSerialize, xcdr_serializer::Xcdr1LeSerializer},
};
use std::io::Write;

const PL_CDR_LE: [u8; 2] = [0x00, 0x03];
const REPRESENTATION_OPTIONS: [u8; 2] = [0x00, 0x00];
const PID_SENTINEL: ParameterId = 1;

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

    pub fn write<T>(&mut self, id: ParameterId, value: &T) -> Result<(), RtpsError>
    where
        T: XTypesSerialize,
    {
        let data_len = Xcdr1LeSerializer::bytes_len(value)?;
        let padded_length = (data_len + 3) & !3;
        if padded_length > u16::MAX as usize {
            return Err(RtpsError::new(RtpsErrorKind::InvalidData, format!("Serialized parameter ID {} with serialized size {} exceeds maximum parameter size of {}", id, padded_length, u16::MAX)));
        }
        self.writer.write_all(&id.to_le_bytes())?;
        self.writer
            .write_all(&(padded_length as u16).to_le_bytes())?;
        value.serialize(&mut Xcdr1LeSerializer::new(&mut self.writer))?;
        const ZEROS: [u8; 4] = [0; 4];
        self.writer.write_all(&ZEROS[..padded_length - data_len])?;
        Ok(())
    }

    pub fn write_with_default<T>(
        &mut self,
        id: ParameterId,
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

    pub fn write_collection<T>(
        &mut self,
        id: ParameterId,
        value_list: &[T],
    ) -> Result<(), RtpsError>
    where
        T: XTypesSerialize,
    {
        for value in value_list {
            self.write(id, value)?;
        }
        Ok(())
    }
}
