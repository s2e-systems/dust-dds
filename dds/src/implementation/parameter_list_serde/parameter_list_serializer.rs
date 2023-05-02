use crate::{
    implementation::data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
    infrastructure::error::{DdsError, DdsResult},
    topic_definition::type_support::{PL_CDR_LE, REPRESENTATION_OPTIONS},
};

use serde::Serialize;

pub struct ParameterListSerializer<W>
where
    W: std::io::Write,
{
    serializer: cdr::Serializer<W, byteorder::LittleEndian>,
}

impl<W> ParameterListSerializer<W>
where
    W: std::io::Write,
{
    pub fn new(writer: W) -> Self {
        Self {
            serializer: cdr::Serializer::new(writer),
        }
    }

    pub fn serialize_payload_header(&mut self) -> DdsResult<()> {
        PL_CDR_LE
            .serialize(&mut self.serializer)
            .map_err(|err| DdsError::PreconditionNotMet(err.to_string()))?;
        REPRESENTATION_OPTIONS
            .serialize(&mut self.serializer)
            .map_err(|err| DdsError::PreconditionNotMet(err.to_string()))?;
        Ok(())
    }

    pub fn serialize_parameter<T>(&mut self, parameter_id: u16, value: &T) -> DdsResult<()>
    where
        T: serde::Serialize,
    {
        let length_without_padding = cdr::size::calc_serialized_data_size(value) as i16;
        let padding_length = (4 - length_without_padding) & 3;
        let length = length_without_padding + padding_length;

        parameter_id
            .serialize(&mut self.serializer)
            .map_err(|err| DdsError::PreconditionNotMet(err.to_string()))?;

        length
            .serialize(&mut self.serializer)
            .map_err(|err| DdsError::PreconditionNotMet(err.to_string()))?;

        value
            .serialize(&mut self.serializer)
            .map_err(|err| DdsError::PreconditionNotMet(err.to_string()))?;

        for _ in 0..padding_length {
            0_u8.serialize(&mut self.serializer)
                .map_err(|err| DdsError::PreconditionNotMet(err.to_string()))?;
        }
        Ok(())
    }

    pub fn serialize_parameter_if_not_default<T>(
        &mut self,
        parameter_id: u16,
        value: &T,
    ) -> DdsResult<()>
    where
        T: serde::Serialize + PartialEq + Default,
    {
        if value != &T::default() {
            self.serialize_parameter::<T>(parameter_id, value)?;
        }
        Ok(())
    }

    pub fn serialize_parameter_vector<T>(&mut self, parameter_id: u16, value: &[T]) -> DdsResult<()>
    where
        T: serde::Serialize,
    {
        for value_i in value {
            self.serialize_parameter(parameter_id, value_i)?;
        }
        Ok(())
    }

    pub fn serialize_sentinel(&mut self) -> DdsResult<()> {
        PID_SENTINEL
            .serialize(&mut self.serializer)
            .map_err(|err| DdsError::PreconditionNotMet(err.to_string()))?;
        [0_u8, 0]
            .serialize(&mut self.serializer)
            .map_err(|err| DdsError::PreconditionNotMet(err.to_string()))?;
        Ok(())
    }
}
