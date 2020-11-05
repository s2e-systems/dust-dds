use crate::messages::submessages::Data;
use crate::messages::submessages::SubmessageHeader;
use crate::messages::submessages::submessage_elements::ParameterList;

use super::{UdpPsmMappingResult, SizeSerializer};
use super::submessage_elements::{serialize_ushort, deserialize_ushort, serialize_entity_id, deserialize_entity_id, serialize_sequence_number, deserialize_sequence_number, serialize_parameter_list, deserialize_parameter_list, serialize_serialized_data, deserialize_serialized_data};

pub fn serialize_data(data: &Data, writer: &mut impl std::io::Write) -> UdpPsmMappingResult<()> {
    let endianness = data.endianness_flag().into();
    let extra_flags = 0;

    let mut to_inline_qos_size_serializer = SizeSerializer::new();
    serialize_entity_id(&data.reader_id(), &mut to_inline_qos_size_serializer)?;
    serialize_entity_id(&data.writer_id(), &mut to_inline_qos_size_serializer)?;
    serialize_sequence_number(&data.writer_sn(), &mut to_inline_qos_size_serializer, endianness)?;

    let octecs_to_inline_qos_size = to_inline_qos_size_serializer.get_size() as u16;

    serialize_ushort(&extra_flags, writer, endianness)?;
    serialize_ushort(&octecs_to_inline_qos_size, writer, endianness)?;

    serialize_entity_id(&data.reader_id(), writer)?;
    serialize_entity_id(&data.writer_id(), writer)?;
    serialize_sequence_number(&data.writer_sn(), writer, endianness)?; 
    
    if data.inline_qos_flag() {
        serialize_parameter_list(data.inline_qos(), writer, endianness)?;
    }

    if data.data_flag() || data.key_flag() {
        serialize_serialized_data(data.serialized_payload(), writer)?;
    }

    Ok(())
}

pub fn deserialize_data(bytes: &[u8], header: SubmessageHeader) -> UdpPsmMappingResult<Data> { 
    let flags = header.flags();

    // X|X|X|N|K|D|Q|E
    /*E*/ let endianness_flag = flags[0];
    /*Q*/ let inline_qos_flag = flags[1];
    /*D*/ let data_flag = flags[2];
    /*K*/ let key_flag = flags[3];
    /*N*/ let non_standard_payload_flag = flags[4];

    let endianness = endianness_flag.into();

    // const HEADER_SIZE : usize = 8;
    let octets_to_inline_qos = usize::from(deserialize_ushort(&bytes[2..4], endianness)?) ; 
    let reader_id = deserialize_entity_id(&bytes[4..8])?;        
    let writer_id = deserialize_entity_id(&bytes[8..12])?;
    let writer_sn = deserialize_sequence_number(&bytes[12..20], endianness)?;
    let (inline_qos, inline_qos_octets) = if inline_qos_flag {
        let inline_qos = deserialize_parameter_list(&bytes[octets_to_inline_qos + 4..], endianness)?; /*Start position is octets_to_inline_qos_value plus extra flags and octets_to_inline_qos_value*/
        let inline_qos_octets = inline_qos.as_bytes(endianness.into()).len(); /* TODO: Very ineficient. Should be improved */
        (inline_qos, inline_qos_octets)
    } else { 
        let inline_qos = ParameterList{parameter:Vec::new()};
        (inline_qos, 0)
    };
    let serialized_payload = if data_flag || key_flag || non_standard_payload_flag {
        let octets_to_serialized_payload = octets_to_inline_qos + 4 + inline_qos_octets;
        deserialize_serialized_data(&bytes[octets_to_serialized_payload..])?
    } else {
        deserialize_serialized_data(&[])?
    };


    Ok(Data::from_raw_parts(endianness_flag, inline_qos_flag, data_flag, key_flag, non_standard_payload_flag, reader_id, writer_id, writer_sn, inline_qos, serialized_payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::types::KeyHash;
    use crate::messages::Endianness;
    use crate::messages::submessages::data_submessage::Payload;
    use crate::messages::submessages::Submessage;
    use crate::types::constants::{ENTITYID_UNKNOWN, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, };
    

    #[test]
    fn test_serialize_data_submessage_without_inline_qos_empty_data() {
        let data = Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN, 
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, 
            1, 
            None,
            Payload::None);
        
        let expected = vec![
            // 0x15_u8, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00,  16, 0x0, // ExtraFlags, octetsToInlineQos (little indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
        ];
        let mut result = Vec::new();
        serialize_data(&data, &mut result).unwrap();
        assert_eq!(expected, result);

        let data_deserialzied = deserialize_data(&result, data.submessage_header(expected.len() as u16)).unwrap();
        assert_eq!(data_deserialzied, data);
    }

    #[test]
    fn test_serialize_data_with_inline_qos_without_data() {
        let key_hash = KeyHash([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]);
        let mut inline_qos = ParameterList::new();
        inline_qos.push(key_hash);
        
        let data = Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN, 
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, 
            1, 
            Some(inline_qos), 
            Payload::None);
        let expected = vec![
            // 0x15_u8, 0b00000011, 44, 0x0, // Submessgae Header
            0x00, 0x00,  16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
            0x70, 0x00, 0x10, 0x00, // [Inline QoS] parameterId, length
            1, 2, 3, 4,             // [Inline QoS] Key hash
            5, 6, 7, 8,             // [Inline QoS] Key hash
            9, 10, 11, 12,          // [Inline QoS] Key hash
            13, 14, 15, 16,         // [Inline QoS] Key hash
            0x01, 0x00, 0x00, 0x00  // [Inline QoS] PID_SENTINEL
        ];
        let mut result = Vec::new();
        serialize_data(&data,&mut result).unwrap();
        assert_eq!(expected, result);

        let data_deserialzied = deserialize_data(&result, data.submessage_header(expected.len() as u16)).unwrap();
        assert_eq!(data_deserialzied, data);
    }

    #[test]
    fn test_serialize_data_submessage_with_inline_qos_with_data() {
        let key_hash = KeyHash([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]);
        let mut inline_qos = ParameterList::new();
        inline_qos.push(key_hash);

        let data = Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN, 
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, 
            1, 
            Some(inline_qos), 
            Payload::Data(vec![1_u8, 2, 3]));
            
        let expected = vec![
            // 0x15_u8, 0b00000111, 47, 0x0, // Submessgae Header
            0x00, 0x00,  16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
            0x70, 0x00, 0x10, 0x00, // [Inline QoS] parameterId, length
            1, 2, 3, 4,             // [Inline QoS] Key hash
            5, 6, 7, 8,             // [Inline QoS] Key hash
            9, 10, 11, 12,          // [Inline QoS] Key hash
            13, 14, 15, 16,         // [Inline QoS] Key hash
            0x01, 0x00, 0x00, 0x00, // [Inline QoS] PID_SENTINEL
            1, 2, 3,             // [Serialized Payload]
        ];
        let mut result = Vec::new();
        serialize_data(&data, &mut result).unwrap();
        assert_eq!(expected, result);

        let data_deserialzied = deserialize_data(&result, data.submessage_header(expected.len() as u16)).unwrap();
        assert_eq!(data_deserialzied, data);
    }
}
