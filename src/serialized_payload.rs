use std::convert::TryInto;

use crate::messages::{ParameterList, Endianness };
use crate::messages::parameter_list::{Pid, Parameter};


#[derive(Debug, Copy, Clone)]
pub enum CdrEndianness {
    LittleEndian,
    BigEndian,
}

impl From<Endianness> for CdrEndianness {
    fn from(value: Endianness) -> Self {
        match value {
            Endianness::LittleEndian => CdrEndianness::LittleEndian,
            Endianness::BigEndian => CdrEndianness::BigEndian,
        }
    }
}

impl From<bool> for CdrEndianness {
    fn from(value: bool) -> Self {
        let endianness: Endianness = value.into();
        endianness.into()
    }
}

impl From<CdrEndianness> for Endianness {
    fn from(value: CdrEndianness) -> Self {
        match value {
            CdrEndianness::LittleEndian => Endianness::LittleEndian,
            CdrEndianness::BigEndian => Endianness::BigEndian,
        }
    }
}

#[derive(PartialEq, Debug)]
struct RepresentationIdentifier([u8; 2]);

#[derive(PartialEq, Debug)]
struct RepresentationOptions([u8; 2]);

#[derive(PartialEq, Debug)]
struct SerializedPayloadHeader {
    representation_identifier: RepresentationIdentifier,
    representation_options: RepresentationOptions,
}

pub struct CdrParameterList {
    endianness: CdrEndianness,
    parameter_list: ParameterList,
}

impl CdrParameterList {
    pub fn new(endianness: CdrEndianness) -> Self {
        Self {
            endianness,
            parameter_list: ParameterList::new(),
        }
    }

    pub fn serialize(&self, writer: &mut impl std::io::Write) {
        // Start by writing the header which depends on the endianness
        match self.endianness {
            CdrEndianness::BigEndian => writer.write(&[0x00, 0x02, 0x00, 0x00]),
            CdrEndianness::LittleEndian => writer.write(&[0x00, 0x03, 0x00, 0x00]),
        }.unwrap();

        for parameter in self.parameter_list.parameter().iter() {
            match self.endianness {
                CdrEndianness::LittleEndian => {
                    writer.write(&parameter.parameter_id().to_le_bytes()).unwrap();
                    writer.write(&parameter.length().to_le_bytes()).unwrap();
                },
                CdrEndianness::BigEndian => {
                    writer.write(&parameter.parameter_id().to_be_bytes()).unwrap();
                    writer.write(&parameter.length().to_be_bytes()).unwrap();
                }
            };

            writer.write(parameter.value(self.endianness).as_slice()).unwrap();
            let padding = parameter.length() as usize - parameter.value(self.endianness).len();
            for _ in 0..padding {
                writer.write(&[0_u8]).unwrap();
            }
        }

        match self.endianness {
            CdrEndianness::BigEndian => writer.write(&ParameterList::PID_SENTINEL.to_be_bytes()).unwrap(),
            CdrEndianness::LittleEndian => writer.write(&ParameterList::PID_SENTINEL.to_le_bytes()).unwrap(),
        };
        writer.write(&[0,0]).unwrap(); // Sentinel length 0
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        if bytes.len() < 4 {
            panic!("Message too small");
        }

        let endianness = match &bytes[0..4] {
            &[0x00, 0x02, 0x00, 0x00] => CdrEndianness::BigEndian,
            &[0x00, 0x03, 0x00, 0x00] => CdrEndianness::LittleEndian,
            _ => panic!("Invalid header"),
        };

        let mut parameter_start_index: usize = 4;
        let mut parameter_list = ParameterList::new();
        loop {
            let (parameter_id, length) = match endianness {
                CdrEndianness::BigEndian => {
                    let parameter_id = i16::from_be_bytes(bytes[parameter_start_index..parameter_start_index+2].try_into().unwrap());
                    let length = i16::from_be_bytes(bytes[parameter_start_index+2..parameter_start_index+4].try_into().unwrap());
                    (parameter_id, length)
                },
                CdrEndianness::LittleEndian => {
                    let parameter_id = i16::from_le_bytes(bytes[parameter_start_index..parameter_start_index+2].try_into().unwrap());
                    let length = i16::from_le_bytes(bytes[parameter_start_index+2..parameter_start_index+4].try_into().unwrap());
                    (parameter_id, length)
                },
            };

            if parameter_id == ParameterList::PID_SENTINEL {
                break;
            }     

            let bytes_end = parameter_start_index + (length + 4) as usize;
            let value = Vec::from(&bytes[parameter_start_index+4..bytes_end]);
            parameter_start_index = bytes_end;

            parameter_list.push(Parameter::new(parameter_id, value));
        }

        Self {
            endianness,
            parameter_list,
        }
    }

    pub fn push<T: Pid + serde::Serialize + std::fmt::Debug + 'static>(&mut self, value: T) {
        self.parameter_list.push(value);
    }

    pub fn find<'de, T>(&self) -> Option<T>
        where T: Pid + serde::Deserialize<'de>
    {
        self.parameter_list.find(self.endianness)
    }

    pub fn find_all<'de, T>(&self) -> Vec<T>
        where T: Pid + serde::Deserialize<'de>
    {
        self.parameter_list.find_all(self.endianness)
    }
}

#[derive(PartialEq, Debug)]
struct StandardSerializedPayload {
    header: SerializedPayloadHeader,
    data: Vec<u8>,
}

// impl RtpsSerialize for StandardSerializedPayload {
//     fn serialize(&self, _writer: &mut impl std::io::Write, _endianness: Endianness) -> RtpsSerdesResult<()> { todo!() }
//     fn octets(&self) -> usize { todo!() }
// }

// impl RtpsDeserialize for StandardSerializedPayload {
//     fn deserialize(_bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> { 
//         todo!() 
//     }
// }



#[derive(PartialEq, Debug)]
pub struct SerializedPayload(pub Vec<u8>);

// impl RtpsSerialize for SerializedPayload {
//     fn serialize(&self, writer: &mut impl std::io::Write, _endianness: Endianness) -> RtpsSerdesResult<()> {
//         writer.write(self.0.as_slice())?;
//         Ok(())
//     }
// }

// impl RtpsDeserialize for SerializedPayload {
//     fn deserialize(bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> {
//         Ok(SerializedPayload(Vec::from(bytes)))
//     }
// }

