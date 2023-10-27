use crate::implementation::type_support::cdr_deserializer::CdrDataDeserializer;

use super::{deserialize::CdrDeserialize, deserializer::CdrDeserializer, error::CdrResult};

#[derive(Clone)]
struct Parameter<'a> {
    parameter_id: i16,
    length: u16,
    data: &'a [u8],
}

impl<'a> Parameter<'a> {
    fn data(&self) -> &[u8] {
        self.data
    }
}

impl<'a, 'de> CdrDeserialize<'de> for Parameter<'a> {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> CdrResult<Self> {
        todo!()
    }
}

pub struct ParameterList<'a> {
    parameter_list: Vec<Parameter<'a>>,
}

impl<'a> ParameterList<'a> {
    pub fn get<T>(&self, id: i16) -> CdrResult<T>
    where
        T: CdrDeserialize<'a>,
    {
        match self.parameter_list.iter().find(|p| p.parameter_id == id) {
            Some(p) => CdrDeserialize::deserialize(&mut CdrDataDeserializer::<
                byteorder::LittleEndian,
            >::new(p.data)),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Parameter ID {} not found in parameter list", id),
            )),
        }
    }

    pub fn get_with_default<T>(&self, id: i16, default: T) -> CdrResult<T>
    where
        T: CdrDeserialize<'a>,
    {
        self.parameter_list
            .iter()
            .find(|p| p.parameter_id == id)
            .map_or(Ok(default), |p| {
                CdrDeserialize::deserialize(
                    &mut CdrDataDeserializer::<byteorder::LittleEndian>::new(p.data),
                )
            })
    }

    pub fn get_all<T>(&self, id: i16) -> CdrResult<Vec<T>>
    where
        T: CdrDeserialize<'a>,
    {
        let mut value_list = Vec::new();
        for p in self.parameter_list.iter().filter(|p| p.parameter_id == id) {
            let value = CdrDeserialize::deserialize(&mut CdrDataDeserializer::<
                byteorder::LittleEndian,
            >::new(p.data))?;
            value_list.push(value);
        }
        Ok(value_list)
    }
}

impl<'a, 'de> CdrDeserialize<'de> for ParameterList<'a> {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> CdrResult<Self> {
        todo!()
    }
}
