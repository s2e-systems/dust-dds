use std::io::Write;

use super::{
    endianness::CdrEndianness, parameter_list_serialize::ParameterListSerialize,
    parameter_list_serializer::ParameterListSerializer, serialize::CdrSerialize,
    serializer::CdrSerializer,
};

type RepresentationIdentifier = [u8; 2];
type RepresentationOptions = [u8; 2];

const CDR_BE: RepresentationIdentifier = [0x00, 0x00];
const CDR_LE: RepresentationIdentifier = [0x00, 0x01];
const PL_CDR_BE: RepresentationIdentifier = [0x00, 0x02];
const PL_CDR_LE: RepresentationIdentifier = [0x00, 0x03];
const REPRESENTATION_OPTIONS: RepresentationOptions = [0x00, 0x00];

pub fn serialize_rtps_cdr(
    value: &impl CdrSerialize,
    writer: &mut Vec<u8>,
    endianness: CdrEndianness,
) -> Result<(), std::io::Error> {
    match endianness {
        CdrEndianness::LittleEndian => writer.write_all(&CDR_LE)?,
        CdrEndianness::BigEndian => writer.write_all(&CDR_BE)?,
    }
    writer.write_all(&REPRESENTATION_OPTIONS)?;
    let mut serializer = CdrSerializer::new(writer, endianness);
    CdrSerialize::serialize(value, &mut serializer)
}

pub fn serialize_rtps_cdr_pl(
    value: &impl ParameterListSerialize,
    writer: &mut Vec<u8>,
    endianness: CdrEndianness,
) -> Result<(), std::io::Error> {
    match endianness {
        CdrEndianness::LittleEndian => writer.write_all(&PL_CDR_LE)?,
        CdrEndianness::BigEndian => writer.write_all(&PL_CDR_BE)?,
    }
    writer.write_all(&REPRESENTATION_OPTIONS)?;
    let mut serializer = ParameterListSerializer::new(writer, endianness);
    ParameterListSerialize::serialize(value, &mut serializer)
}
