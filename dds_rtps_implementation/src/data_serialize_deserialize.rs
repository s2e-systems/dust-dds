use std::{io::{BufRead, Read, Write}, marker::PhantomData};

use byteorder::{ByteOrder, ReadBytesExt};
use cdr::Serializer;

use crate::dds_type::{Endianness, BigEndian, LittleEndian};

pub trait MappingWriteByteOrdered {
    fn write_ordered<W: Write, E: Endianness>(
        &self,
        writer: W,
    ) -> std::result::Result<(), std::io::Error>;
}

impl<T> MappingWriteByteOrdered for T
where
    T: serde::Serialize,
{
    fn write_ordered<W: Write, E: Endianness>(
        &self,
        writer: W,
    ) -> std::result::Result<(), std::io::Error> {
        let mut serializer = Serializer::<_, E::Endianness>::new(writer);
        serde::Serialize::serialize(self, &mut serializer)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }
}

#[derive(Debug, PartialEq)]
struct Parameter<T> {
    parameter_id: u16,
    value: T,
}

impl<T: serde::Serialize> Parameter<T> {
    fn new(parameter_id: u16, value: T) -> Self {
        Self {
            parameter_id,
            value,
        }
    }
}

impl<T: serde::Serialize> MappingWriteByteOrdered for Parameter<T> {
    fn write_ordered<W: Write, E: Endianness>(
        &self,
        mut writer: W,
    ) -> std::result::Result<(), std::io::Error> {
        let length_without_padding = (cdr::calc_serialized_size(&self.value) - 4) as i16;
        let padding: &[u8] = match length_without_padding % 4 {
            1 => &[0; 3],
            2 => &[0; 2],
            3 => &[0; 1],
            _ => &[],
        };
        let length = length_without_padding + padding.len() as i16;
        self.parameter_id.write_ordered::<_, E>(&mut writer)?;
        length.write_ordered::<_, E>(&mut writer)?;
        let mut serializer = cdr::Serializer::<_, E::Endianness>::new(&mut writer);
        self.value.serialize(&mut serializer).map_err(|err| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, err.to_string())
        })?;
        writer.write_all(padding)
    }
}

const PID_SENTINEL: u16 = 1;

pub struct ParameterListSerialize(Vec<Parameter<Box<dyn erased_serde::Serialize>>>);
impl MappingWriteByteOrdered for ParameterListSerialize {
    fn write_ordered<W: Write, E: Endianness>(
        &self,
        mut writer: W,
    ) -> std::result::Result<(), std::io::Error> {
        writer.write(&E::REPRESENTATION_IDENTIFIER).unwrap();
        writer.write(&E::REPRESENTATION_OPTIONS).unwrap();
        for parameter_i in &self.0 {
            parameter_i.write_ordered::<_, E>(&mut writer).unwrap();
        }

        Ok(())
    }
}

pub struct ParameterSerializer<W, E>
where
    W: std::io::Write,
    E: Endianness,
{
    writer: W,
    phantom: PhantomData<E>,
}

impl<W, E> ParameterSerializer<W, E>
where
    W: std::io::Write,
    E: Endianness,
{
    pub fn new(mut writer: W) -> Self {
        writer.write(&E::REPRESENTATION_IDENTIFIER).unwrap();
        writer.write(&E::REPRESENTATION_OPTIONS).unwrap();

        Self {
            writer,
            phantom: PhantomData,
        }
    }

    pub fn serialize_parameter<T>(
        &mut self,
        parameter_id: u16,
        value: &T,
    ) -> std::result::Result<(), std::io::Error>
    where
        T: serde::Serialize,
    {
        Parameter::new(parameter_id, value).write_ordered::<_, E>(&mut self.writer)
    }
}

impl<W, E> Drop for ParameterSerializer<W, E>
where
    W: std::io::Write,
    E: Endianness,
{
    fn drop(&mut self) {
        PID_SENTINEL
            .write_ordered::<_, E>(&mut self.writer)
            .unwrap();
        [0_u8, 0].write_ordered::<_, E>(&mut self.writer).unwrap();
    }
}


pub trait MappingRead<'de>: Sized {
    fn read(buf: &mut &'de [u8]) -> Result<Self, std::io::Error>;
}


impl<'de: 'a, 'a> Parameter<&'a [u8]> {
    fn read<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, std::io::Error> {
        let parameter_id = buf.read_u16::<B>()?;
        let length = buf.read_i16::<B>()?;
        let (value, following) = buf.split_at(length as usize);
        *buf = following;
        Ok(Self {
            parameter_id,
            value,
        })
    }
}

#[derive(Debug, PartialEq)]
enum RepresentationIdentifier {
    PlCdrBe,
    PlCdrLe,
}

impl<'de> MappingRead<'de> for RepresentationIdentifier {
    fn read(buf: &mut &'de [u8]) -> Result<Self, std::io::Error> {
        let mut representation_identifier = [0; 2];
        buf.read(&mut representation_identifier)?;
        match representation_identifier {
            BigEndian::REPRESENTATION_IDENTIFIER => Ok(RepresentationIdentifier::PlCdrBe),
            LittleEndian::REPRESENTATION_IDENTIFIER => Ok(RepresentationIdentifier::PlCdrLe),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid representation identifier",
            )),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ParameterList<'a> {
    parameter: Vec<Parameter<&'a [u8]>>,
    representation_identifier: RepresentationIdentifier,
}

impl<'de: 'a, 'a> MappingRead<'de> for ParameterList<'a> {
    fn read(buf: &mut &'de [u8]) -> Result<Self, std::io::Error> {
        let representation_identifier = MappingRead::read(buf)?;
        // ignore representation_options
        buf.consume(2);

        let mut parameter = vec![];
        loop {
            let parameter_i = match representation_identifier {
                RepresentationIdentifier::PlCdrBe => {
                    Parameter::<&[u8]>::read::<byteorder::BigEndian>(buf)?
                }
                RepresentationIdentifier::PlCdrLe => {
                    Parameter::<&[u8]>::read::<byteorder::LittleEndian>(buf)?
                }
            };
            if parameter_i.parameter_id == PID_SENTINEL {
                break;
            } else {
                parameter.push(parameter_i);
            }
        }
        Ok(Self {
            parameter,
            representation_identifier,
        })
    }
}


impl<'de> ParameterList<'de> {
    pub fn get<T: serde::Deserialize<'de>>(&self, parameter_id: u16) -> Result<T, std::io::Error> {
        for parameter in self.parameter.iter() {
            if parameter.parameter_id == parameter_id {
                return Ok(self.deserialize_parameter(parameter)?);
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Parameter with id {} not found", parameter_id),
        ))
    }

    pub fn get_list<T: serde::Deserialize<'de>>(
        &self,
        parameter_id: u16,
    ) -> Result<Vec<T>, std::io::Error> {
        let mut result = vec![];
        for parameter in self.parameter.iter() {
            if parameter.parameter_id == parameter_id {
                if let Ok(result_i) = self.deserialize_parameter(parameter) {
                    result.push(result_i);
                }
            }
        }
        Ok(result)
    }

    fn deserialize_parameter<T: serde::Deserialize<'de>>(
        &self,
        parameter: &Parameter<&[u8]>,
    ) -> Result<T, std::io::Error> {
        Ok(match self.representation_identifier {
            RepresentationIdentifier::PlCdrBe => {
                let mut deserializer = cdr::Deserializer::<_, _, byteorder::BigEndian>::new(
                    parameter.value,
                    cdr::Infinite,
                );
                serde::Deserialize::deserialize(&mut deserializer).map_err(|err| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
                })?
            }
            RepresentationIdentifier::PlCdrLe => {
                let mut deserializer = cdr::Deserializer::<_, _, byteorder::LittleEndian>::new(
                    parameter.value,
                    cdr::Infinite,
                );
                serde::Deserialize::deserialize(&mut deserializer).map_err(|err| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())
                })?
            }
        })
    }
}