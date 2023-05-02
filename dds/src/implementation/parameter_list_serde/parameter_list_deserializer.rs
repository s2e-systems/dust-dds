use std::marker::PhantomData;

use byteorder::{ByteOrder, ReadBytesExt};

use crate::{
    implementation::data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
    infrastructure::error::{DdsError, DdsResult},
};

#[derive(Debug, PartialEq)]
struct Parameter<'a> {
    parameter_id: u16,
    value: &'a [u8],
}

impl<'de: 'a, 'a> Parameter<'a> {
    fn read<E: ByteOrder>(buf: &mut &'de [u8]) -> DdsResult<Self> {
        let parameter_id = buf
            .read_u16::<E>()
            .map_err(|err| DdsError::PreconditionNotMet(err.to_string()))?;
        let length = buf
            .read_i16::<E>()
            .map_err(|err| DdsError::PreconditionNotMet(err.to_string()))?;
        let (value, following) = buf.split_at(length as usize);
        *buf = following;
        Ok(Self {
            parameter_id,
            value,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct ParameterListDeserializer<'a, E> {
    parameter: Vec<Parameter<'a>>,
    phantom: PhantomData<E>,
}

impl<'de: 'a, 'a, E: ByteOrder> ParameterListDeserializer<'a, E> {
    pub fn read(buf: &mut &'de [u8]) -> DdsResult<Self> {
        let mut parameter = vec![];
        loop {
            let parameter_i = Parameter::read::<E>(buf)?;
            if parameter_i.parameter_id == PID_SENTINEL {
                break;
            } else {
                parameter.push(parameter_i);
            }
        }
        Ok(Self {
            parameter,
            phantom: PhantomData,
        })
    }
}

impl<'de, E: ByteOrder> ParameterListDeserializer<'de, E> {
    pub fn get<T>(&self, parameter_id: u16) -> DdsResult<T>
    where
        T: serde::Deserialize<'de>,
    {
        for parameter in self.parameter.iter() {
            if parameter.parameter_id == parameter_id {
                return self.deserialize_parameter::<T>(parameter);
            }
        }
        Err(DdsError::PreconditionNotMet(format!(
            "Parameter with id {:#06x} not found",
            parameter_id
        )))
    }
    pub fn get_or_default<T>(&self, parameter_id: u16) -> DdsResult<T>
    where
        T: serde::Deserialize<'de> + Default,
    {
        for parameter in &self.parameter {
            if parameter.parameter_id == parameter_id {
                return self.deserialize_parameter::<T>(parameter);
            }
        }
        Ok(T::default())
    }

    pub fn get_list<T>(&self, parameter_id: u16) -> DdsResult<Vec<T>>
    where
        T: serde::Deserialize<'de>,
    {
        let mut result = vec![];
        for parameter in &self.parameter {
            if parameter.parameter_id == parameter_id {
                result.push(self.deserialize_parameter::<T>(parameter)?);
            }
        }
        Ok(result)
    }

    fn deserialize_parameter<T: serde::Deserialize<'de>>(
        &self,
        parameter: &Parameter,
    ) -> DdsResult<T> {
        let mut deserializer = cdr::Deserializer::<_, _, E>::new(parameter.value, cdr::Infinite);
        serde::Deserialize::deserialize(&mut deserializer).map_err(|err| {
            DdsError::PreconditionNotMet(format!("deserialize_parameter failed with: {}", err))
        })
    }
}
