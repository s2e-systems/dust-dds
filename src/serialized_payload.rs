use crate::messages::{ParameterList, SubmessageElement, Endianness, Pid};

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
    endianness: Endianness,
    parameter_list: ParameterList,
}

impl CdrParameterList {
    pub fn new(endianness: Endianness) -> Self {
        Self {
            endianness,
            parameter_list: ParameterList::new(),
        }
    }

    pub fn serialize(&self, writer: &mut impl std::io::Write) {
        // Start by writing the header which depends on the endianness
        match self.endianness {
            Endianness::BigEndian => writer.write(&[0x00, 0x02, 0x00, 0x00]),
            Endianness::LittleEndian => writer.write(&[0x00, 0x03, 0x00, 0x00]),
        }.unwrap();

        self.parameter_list.serialize(writer, self.endianness).unwrap();
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        if bytes.len() < 4 {
            panic!("Message too small");
        }

        let endianness = match &bytes[0..4] {
            &[0x00, 0x02, 0x00, 0x00] => Endianness::BigEndian,
            &[0x00, 0x03, 0x00, 0x00] => Endianness::LittleEndian,
            _ => panic!("Invalid header"),
        };

        let parameter_list = ParameterList::deserialize(&bytes[4..], endianness).unwrap();

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
        self.parameter_list.find_all(self.endianness).unwrap()
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

