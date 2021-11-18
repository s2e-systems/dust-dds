use std::marker::PhantomData;

use rust_dds_api::return_type::{DDSError, DDSResult};
use serde::Serialize;

use crate::dds_type::Endianness;

use super::parameter_id_values::PID_SENTINEL;

pub struct ParameterListSerializer<W, E>
where
    W: std::io::Write,
    E: Endianness,
{
    serializer: cdr::Serializer<W, E::Endianness>,
    phantom: PhantomData<E>,
}

impl<W, E> ParameterListSerializer<W, E>
where
    W: std::io::Write,
    E: Endianness,
{
    pub fn new(writer: W) -> Self {
        Self {
            serializer: cdr::Serializer::<_, E::Endianness>::new(writer),
            phantom: PhantomData,
        }
    }

    pub fn serialize_payload_header(&mut self) -> DDSResult<()> {
        E::REPRESENTATION_IDENTIFIER
            .serialize(&mut self.serializer)
            .map_err(|err| DDSError::PreconditionNotMet(err.to_string()))?;
        E::REPRESENTATION_OPTIONS
            .serialize(&mut self.serializer)
            .map_err(|err| DDSError::PreconditionNotMet(err.to_string()))?;
        Ok(())
    }

    pub fn serialize_parameter<'a, T, U>(
        &mut self,
        parameter_id: u16,
        value: &'a U,
    ) -> DDSResult<()>
    where
        T: serde::Serialize + From<&'a U>,
        U: 'a,
    {
        let serializale_value = &T::from(value);
        let length_without_padding = cdr::size::calc_serialized_data_size(serializale_value) as i16;
        let padding_length = (4 - length_without_padding) & 3;
        let length = length_without_padding + padding_length;

        parameter_id
            .serialize(&mut self.serializer)
            .map_err(|err| DDSError::PreconditionNotMet(err.to_string()))?;

        length
            .serialize(&mut self.serializer)
            .map_err(|err| DDSError::PreconditionNotMet(err.to_string()))?;

        serializale_value
            .serialize(&mut self.serializer)
            .map_err(|err| DDSError::PreconditionNotMet(err.to_string()))?;

        for _ in 0..padding_length {
            0_u8.serialize(&mut self.serializer)
                .map_err(|err| DDSError::PreconditionNotMet(err.to_string()))?;
        }
        Ok(())
    }

    pub fn serialize_parameter_if_not_default<'a, T, U>(
        &mut self,
        parameter_id: u16,
        value: &'a U,
    ) -> DDSResult<()>
    where
        T: serde::Serialize + From<&'a U>,
        U: PartialEq<U> + Default,
    {
        if value != &U::default() {
            self.serialize_parameter::<T,U>(parameter_id, value)?;
        }
        Ok(())
    }

    pub fn serialize_parameter_vector<'a, T, U>(
        &mut self,
        parameter_id: u16,
        value: &'a Vec<U>,
    ) -> DDSResult<()>
    where
        T: serde::Serialize + From<&'a U>,
    {
        for value_i in value.iter() {
            self.serialize_parameter::<T,U>(parameter_id, value_i)?;
        }
        Ok(())
    }

    pub fn serialize_sentinel(&mut self) -> DDSResult<()> {
        PID_SENTINEL
            .serialize(&mut self.serializer)
            .map_err(|err| DDSError::PreconditionNotMet(err.to_string()))?;
        [0_u8, 0]
            .serialize(&mut self.serializer)
            .map_err(|err| DDSError::PreconditionNotMet(err.to_string()))?;
        Ok(())
    }
}
