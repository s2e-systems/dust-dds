use crate::types::{EntityId, SequenceNumber, InlineQosParameterList};

use super::helpers::{deserialize, endianess, parse_inline_qos_parameter_list, SequenceNumberSerialization};

use super::{Result, ErrorMessage};

#[derive(PartialEq, Debug)]
pub enum Payload {
    None,
    Data(Vec<u8>),
    Key(Vec<u8>),
    NonStandard(Vec<u8>),
}

#[derive(PartialEq, Debug)]
pub struct Data {
    reader_id: EntityId,
    writer_id: EntityId,
    writer_sn: SequenceNumber,
    inline_qos: Option<InlineQosParameterList>,
    serialized_payload: Payload,
}

impl Data {
    pub fn new( reader_id: EntityId, writer_id: EntityId,  writer_sn: SequenceNumber, inline_qos: Option<InlineQosParameterList>, serialized_payload: Payload) -> Data {
        Data {
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        }
    }

    pub fn reader_id(&self) -> &EntityId {
        &self.reader_id
    }

    pub fn writer_id(&self) -> &EntityId {
        &self.writer_id
    }

    pub fn writer_sn(&self) -> &SequenceNumber {
        &self.writer_sn
    }

    pub fn inline_qos(&self) -> &Option<InlineQosParameterList> {
        &self.inline_qos
    }

    pub fn serialized_payload(&self) -> &Payload {
        &self.serialized_payload
    }

    pub fn take(self) -> (EntityId, EntityId, SequenceNumber, Option<InlineQosParameterList>, Payload) {
        (self.reader_id, self.writer_id, self.writer_sn, self.inline_qos, self.serialized_payload)
    }
}

pub fn parse_data_submessage(submessage: &[u8], submessage_flags: &u8) -> Result<Data> {

    const INLINE_QOS_FLAG_MASK: u8 = 0x02;
    const DATA_FLAG_MASK: u8 = 0x04;
    const KEY_FLAG_MASK: u8 = 0x08;
    const NON_STANDARD_PAYLOAD_FLAG_MASK: u8 = 0x10;

    const EXTRA_FLAGS_FIRST_INDEX: usize = 0;
    const EXTRA_FLAGS_LAST_INDEX: usize = 1;
    const OCTETS_TO_INLINE_QOS_FIRST_INDEX: usize = 2;
    const OCTETS_TO_INLINE_QOS_LAST_INDEX: usize = 3;
    const READER_ID_FIRST_INDEX: usize = 4;
    const READER_ID_LAST_INDEX: usize = 7;
    const WRITER_ID_FIRST_INDEX: usize = 8;
    const WRITER_ID_LAST_INDEX: usize = 11;
    const WRITER_SN_FIRST_INDEX: usize = 12;
    const WRITER_SN_LAST_INDEX: usize = 19;

    let submessage_endianess = endianess(submessage_flags)?;
    let inline_qos_flag = submessage_flags & INLINE_QOS_FLAG_MASK == INLINE_QOS_FLAG_MASK;
    let data_flag = submessage_flags & DATA_FLAG_MASK == DATA_FLAG_MASK;
    let key_flag = submessage_flags & KEY_FLAG_MASK == KEY_FLAG_MASK;

    // TODO: Implement non-standard payload
    let _non_standard_payload_flag = submessage_flags & NON_STANDARD_PAYLOAD_FLAG_MASK == NON_STANDARD_PAYLOAD_FLAG_MASK;

    if data_flag && key_flag {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    let extra_flags = deserialize::<u16>(submessage, &EXTRA_FLAGS_FIRST_INDEX, &EXTRA_FLAGS_LAST_INDEX, &submessage_endianess)?;
    if extra_flags != 0 {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    let octecs_to_inline_qos = deserialize::<u16>(submessage, &OCTETS_TO_INLINE_QOS_FIRST_INDEX, &OCTETS_TO_INLINE_QOS_LAST_INDEX, &submessage_endianess)? as usize;

    let reader_id = deserialize::<EntityId>(submessage, &READER_ID_FIRST_INDEX, &READER_ID_LAST_INDEX, &submessage_endianess)?;
    
    let writer_id = deserialize::<EntityId>(submessage, &WRITER_ID_FIRST_INDEX, &WRITER_ID_LAST_INDEX, &submessage_endianess)?;

    let writer_sn : SequenceNumber = deserialize::<SequenceNumberSerialization>(submessage, &WRITER_SN_FIRST_INDEX, &WRITER_SN_LAST_INDEX, &submessage_endianess)?.into();

    // Octets to data is considered as having the same meaning as octets to inline qos,
    // i.e. counting from the byte after the octets to inline qos field

    let (inline_qos, octets_to_data) =
        if inline_qos_flag {
            let inline_qos_first_index = OCTETS_TO_INLINE_QOS_LAST_INDEX + octecs_to_inline_qos + 1;
            let (parameter_list, parameter_list_size) = parse_inline_qos_parameter_list(submessage, &inline_qos_first_index, &submessage_endianess)?;
            let octets_to_data = octecs_to_inline_qos + parameter_list_size;
            (Some(parameter_list), octets_to_data)
        } else {
            (None, octecs_to_inline_qos)
        };

    let payload_first_index = OCTETS_TO_INLINE_QOS_LAST_INDEX + octets_to_data + 1;

    let serialized_payload = 
        if data_flag && !key_flag {
            Payload::Data(submessage[payload_first_index..].to_vec())
        } else if !data_flag && key_flag {
            Payload::Key(submessage[payload_first_index..].to_vec())
        } else {
            Payload::None
        };

    Ok(Data{
        reader_id,
        writer_id,
        writer_sn,
        inline_qos,
        serialized_payload,
    })
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::parser::InlineQosParameter;

    const DATA_SUBMESSAGE_BIG_ENDIAN: [u8;60] =
    [
        0x00, 0x00, 0x00, 0x10,
        0x10, 0x12, 0x14, 0x16,
        0x26, 0x24, 0x22, 0x20,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x04, 0xD1,
        0x00, 0x70, 0x00, 0x10,
        0x01, 0x02, 0x03, 0x04,
        0x05, 0x06, 0x07, 0x08,
        0x09, 0x0A, 0x0B, 0x0C,
        0x0D, 0x0E, 0x0F, 0x10,
        0x00, 0x10, 0x00, 0x08,
        0x10, 0x11, 0x12, 0x13,
        0x14, 0x15, 0x16, 0x17,
        0x00, 0x01, 0x00, 0x00,
        0x20, 0x30, 0x40, 0x50,
    ];

    #[test]
    fn test_parse_data_submessage_without_inline_qos_or_data() {
        let data = parse_data_submessage(&DATA_SUBMESSAGE_BIG_ENDIAN, &0).unwrap();
        assert_eq!(data.reader_id, EntityId::new([0x10,0x12,0x14],0x16));
        assert_eq!(data.writer_id, EntityId::new([0x26,0x24,0x22],0x20));
        assert_eq!(data.writer_sn, 1233);
        assert_eq!(data.inline_qos, None);
        assert_eq!(data.serialized_payload, Payload::None);
    }

    #[test]
    fn test_parse_data_submessage_with_inline_qos_without_data() {
        let data = parse_data_submessage(&DATA_SUBMESSAGE_BIG_ENDIAN, &2).unwrap();
        assert_eq!(data.reader_id, EntityId::new([0x10,0x12,0x14],0x16));
        assert_eq!(data.writer_id, EntityId::new([0x26,0x24,0x22],0x20));
        assert_eq!(data.writer_sn, 1233);
        assert_eq!(data.serialized_payload, Payload::None);
        let inline_qos = data.inline_qos.unwrap();
        assert_eq!(inline_qos.len(),1);
        assert_eq!(inline_qos[0], InlineQosParameter::KeyHash([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,]));
    }

    #[test]
    fn test_parse_data_submessage_with_inline_qos_and_data() {
        let data = parse_data_submessage(&DATA_SUBMESSAGE_BIG_ENDIAN, &6).unwrap();
        assert_eq!(data.reader_id, EntityId::new([0x10,0x12,0x14],0x16));
        assert_eq!(data.writer_id, EntityId::new([0x26,0x24,0x22],0x20));
        assert_eq!(data.writer_sn, 1233);
        let inline_qos = data.inline_qos.unwrap();
        assert_eq!(inline_qos.len(),1);
        assert_eq!(inline_qos[0], InlineQosParameter::KeyHash([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,]));
        if let Payload::Data(serialized_data) = data.serialized_payload {
            assert_eq!(serialized_data, vec!(0x20, 0x30, 0x40, 0x50,));

        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_parse_data_submessage_with_inline_qos_and_serialized_key()
    {
        let data = parse_data_submessage(&DATA_SUBMESSAGE_BIG_ENDIAN, &10).unwrap();
        assert_eq!(data.reader_id, EntityId::new([0x10,0x12,0x14],0x16));
        assert_eq!(data.writer_id, EntityId::new([0x26,0x24,0x22],0x20));
        assert_eq!(data.writer_sn, 1233);
        let inline_qos = data.inline_qos.unwrap();
        assert_eq!(inline_qos.len(),1);
        assert_eq!(inline_qos[0], InlineQosParameter::KeyHash([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,]));
        if let Payload::Key(serialized_data) = data.serialized_payload {
            assert_eq!(serialized_data, vec!(0x20, 0x30, 0x40, 0x50,));

        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_parse_data_submessage_without_inline_qos_with_serialized_data()
    {
        // Parse message considering serialized data and no inline qos
        let data = parse_data_submessage(&DATA_SUBMESSAGE_BIG_ENDIAN, &8).unwrap();
        assert_eq!(data.reader_id, EntityId::new([0x10,0x12,0x14],0x16));
        assert_eq!(data.writer_id, EntityId::new([0x26,0x24,0x22],0x20));
        assert_eq!(data.writer_sn, 1233);
        assert_eq!(data.inline_qos, None);
        if let Payload::Key(serialized_data) = data.serialized_payload {
            assert_eq!(serialized_data, vec!(
                0x00, 0x70, 0x00, 0x10,
                0x01, 0x02, 0x03, 0x04,
                0x05, 0x06, 0x07, 0x08,
                0x09, 0x0A, 0x0B, 0x0C,
                0x0D, 0x0E, 0x0F, 0x10,
                0x00, 0x10, 0x00, 0x08,
                0x10, 0x11, 0x12, 0x13,
                0x14, 0x15, 0x16, 0x17,
                0x00, 0x01, 0x00, 0x00,
                0x20, 0x30, 0x40, 0x50,));
        } else {
            assert!(false);
        }
    }
}