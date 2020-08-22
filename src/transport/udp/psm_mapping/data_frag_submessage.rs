
impl UdpPsmMapping for DataFrag {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        // let sample_size = ULong(0); // TODO: what is sample_size? It is in PSM but nut in PIM. Probably: data_size
        let endianness = Endianness::from(self.endianness_flag);
        let extra_flags = submessage_elements::UShort(0);
        let octecs_to_inline_qos = submessage_elements::UShort((
            self.reader_id.octets() + 
            self.writer_id.octets() + 
            self.writer_sn.octets() + 
            self.fragment_starting_num.octets() + 
            2 /*self.fragments_in_submessage.octets() */+ 
            2 /*self.fragment_size.octets() */+ 
            4 /*self.data_size.octets()) as UShort*/) as u16);
        
        self.submessage_header().compose(writer)?;
        extra_flags.serialize(writer, endianness)?;
        octecs_to_inline_qos.serialize(writer, endianness)?;
        self.reader_id.serialize(writer, endianness)?;
        self.writer_id.serialize(writer, endianness)?;
        self.writer_sn.serialize(writer, endianness)?;
        self.fragment_starting_num.serialize(writer, endianness)?;
        self.fragments_in_submessage.serialize(writer, endianness)?;
        self.fragment_size.serialize(writer, endianness)?;
        self.data_size.serialize(writer, endianness)?;
        if self.inline_qos_flag {
            self.inline_qos.as_ref().unwrap().serialize(writer, endianness)?;
        };
        self.serialized_payload.serialize(writer, endianness)?;
        Ok(())
    }

    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> { 
        let header = SubmessageHeader::parse(bytes)?;
        let flags = header.flags();
        // X|X|X|X|N|K|Q|E
        /*E*/ let endianness_flag = flags[0];
        /*Q*/ let inline_qos_flag = flags[1];
        /*K*/ let key_flag = flags[2];
        /*N*/ let non_standard_payload_flag = flags[3];

        let endianness = Endianness::from(endianness_flag);

        const HEADER_SIZE : usize = 8;
        let octets_to_inline_qos = usize::from(submessage_elements::UShort::deserialize(&bytes[6..8], endianness)?.0) + HEADER_SIZE /* header and extra flags*/;
        let reader_id = submessage_elements::EntityId::deserialize(&bytes[8..12], endianness)?;        
        let writer_id = submessage_elements::EntityId::deserialize(&bytes[12..16], endianness)?;
        let writer_sn = submessage_elements::SequenceNumber::deserialize(&bytes[16..24], endianness)?;
        let fragment_starting_num = submessage_elements::FragmentNumber::deserialize(&bytes[24..28], endianness)?;
        let fragments_in_submessage = submessage_elements::UShort::deserialize(&bytes[28..30], endianness)?;
        let fragment_size = submessage_elements::UShort::deserialize(&bytes[30..32], endianness)?;
        let data_size = submessage_elements::ULong::deserialize(&bytes[32..36], endianness)?;


        let inline_qos = if inline_qos_flag {
            Some(ParameterList::deserialize(&bytes[octets_to_inline_qos..], endianness)?)
        } else { 
            None
        };
        let end_of_submessage = usize::from(header.submessage_length()) + header.octets();
        let octets_to_serialized_payload = octets_to_inline_qos + inline_qos.octets();
        let serialized_payload = submessage_elements::SerializedDataFragment::deserialize(&bytes[octets_to_serialized_payload..end_of_submessage], endianness).ok();
  
        Ok(DataFrag {
            endianness_flag,
            inline_qos_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            fragment_starting_num,
            fragments_in_submessage,
            fragment_size,
            data_size,
            inline_qos, 
            serialized_payload, 
        })
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::inline_qos_types::KeyHash;
    use crate::types::constants::{ENTITYID_UNKNOWN, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, };
    use crate::messages::parameter_list::ParameterList;

    #[test]
    fn parse_data_frag_submessage() {
        let endianness = Endianness::LittleEndian;
        let key_hash = KeyHash([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]);
        let mut inline_qos = ParameterList::new();
        inline_qos.push(key_hash);
        
        let expected = DataFrag {
            endianness_flag: endianness.into(),
            inline_qos_flag: true,
            key_flag: true,
            non_standard_payload_flag: false,
            reader_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            writer_id: submessage_elements::EntityId(ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER),
            writer_sn: submessage_elements::SequenceNumber(1),
            fragment_starting_num: submessage_elements::FragmentNumber(1), 
            fragments_in_submessage: submessage_elements::UShort(2),
            fragment_size: submessage_elements::UShort(3),
            data_size: submessage_elements::ULong(4),
            inline_qos: Some(inline_qos), 
            serialized_payload: Some(submessage_elements::SerializedDataFragment(vec![1, 2, 3])), 
        };

        let bytes = vec![
            0x15_u8, 0b00000111, 59, 0x0, // Submessgae Header
            0x00, 0x00,  28, 0x0, // ExtraFlags, octetsToInlineQos 
            0x00, 0x00, 0x00, 0x00, // readerId 
            0x00, 0x01, 0x00, 0xc2, // writerId
            0x00, 0x00, 0x00, 0x00, // writerSN
            0x01, 0x00, 0x00, 0x00, // writerSN
            1, 0, 0, 0,             // fragmentStartingNum
            2, 0, 3, 0,             // fragmentsInSubmessage | fragmentSize
            4, 0, 0, 0,             // sampleSize
            0x70, 0x00, 0x10, 0x00, // [Inline QoS] parameterId, length
            1, 2, 3, 4,             // [Inline QoS] Key hash
            5, 6, 7, 8,             // [Inline QoS] Key hash
            9, 10, 11, 12,          // [Inline QoS] Key hash
            13, 14, 15, 16,         // [Inline QoS] Key hash
            0x01, 0x00, 0x00, 0x00, // [Inline QoS] PID_SENTINEL
            1, 2, 3,             // [Serialized Payload]
        ];
        let result = DataFrag::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn compose_data_frag_submessage() {
        let endianness = Endianness::LittleEndian;
        let key_hash = KeyHash([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]);
        
        let mut inline_qos = ParameterList::new();
        inline_qos.push(key_hash);

        let message = DataFrag {
            endianness_flag: endianness.into(),
            inline_qos_flag: true,
            key_flag: true,
            non_standard_payload_flag: false,
            reader_id: submessage_elements::EntityId(ENTITYID_UNKNOWN),
            writer_id: submessage_elements::EntityId(ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER),
            writer_sn: submessage_elements::SequenceNumber(1),
            fragment_starting_num: submessage_elements::FragmentNumber(1), 
            fragments_in_submessage: submessage_elements::UShort(2),
            fragment_size: submessage_elements::UShort(3),
            data_size: submessage_elements::ULong(4),
            inline_qos: Some(inline_qos), 
            serialized_payload: Some(submessage_elements::SerializedDataFragment(vec![1, 2, 3])), 
        };

        let expected = vec![
            0x15_u8, 0b00000111, 59, 0x0, // Submessgae Header
            0x00, 0x00,  28, 0x0, // ExtraFlags, octetsToInlineQos 
            0x00, 0x00, 0x00, 0x00, // readerId 
            0x00, 0x01, 0x00, 0xc2, // writerId
            0x00, 0x00, 0x00, 0x00, // writerSN
            0x01, 0x00, 0x00, 0x00, // writerSN
            1, 0, 0, 0,             // fragmentStartingNum
            2, 0, 3, 0,             // fragmentsInSubmessage | fragmentSize
            4, 0, 0, 0,             // sampleSize
            0x70, 0x00, 0x10, 0x00, // [Inline QoS] parameterId, length
            1, 2, 3, 4,             // [Inline QoS] Key hash
            5, 6, 7, 8,             // [Inline QoS] Key hash
            9, 10, 11, 12,          // [Inline QoS] Key hash
            13, 14, 15, 16,         // [Inline QoS] Key hash
            0x01, 0x00, 0x00, 0x00, // [Inline QoS] PID_SENTINEL
            1, 2, 3,             // [Serialized Payload]
        ];
        let mut writer = Vec::new();
        message.compose(&mut writer).unwrap();
        assert_eq!(expected, writer);
    }
}
