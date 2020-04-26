use crate::types::{EntityId, InlineQosParameterList, SequenceNumber, KeyHash};

use super::helpers::{
    deserialize, endianess, parse_inline_qos_parameter_list, SequenceNumberSerialization,
};

use super::{ErrorMessage, Result, SubmessageKind};
use serde::ser::{Serialize, Serializer, SerializeStruct};
use cdr::{PlCdrLe, Infinite, LittleEndian};
use cdr::ser::serialize_data;
use serde_bytes::ByteBuf;

#[derive(PartialEq, Debug)]
pub enum Payload {
    None,
    Data(Vec<u8>),
    Key(Vec<u8>),
    NonStandard(Vec<u8>),
}

#[derive(PartialEq, Debug)]
pub struct Data {
    // status_info: StatusInfo,
    reader_id: EntityId,
    writer_id: EntityId,
    key_hash: KeyHash,
    writer_sn: SequenceNumber,
    inline_qos: Option<InlineQosParameterList>,
    serialized_payload: Payload,
}

impl Data {
    pub fn new(
        reader_id: EntityId,
        writer_id: EntityId,
        key_hash: KeyHash,
        writer_sn: SequenceNumber,
        inline_qos: Option<InlineQosParameterList>,
        serialized_payload: Payload,
    ) -> Data {
        Data {
            reader_id,
            writer_id,
            key_hash,
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

    pub fn key_hash(&self) -> &KeyHash {
        &self.key_hash
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

    pub fn take(
        self,
    ) -> (
        EntityId,
        EntityId,
        SequenceNumber,
        Option<InlineQosParameterList>,
        Payload,
    ) {
        (
            self.reader_id,
            self.writer_id,
            self.writer_sn,
            self.inline_qos,
            self.serialized_payload,
        )
    }
}

const INLINE_QOS_FLAG_MASK: u8 = 0x02;
const DATA_FLAG_MASK: u8 = 0x04;
const KEY_FLAG_MASK: u8 = 0x08;
const NON_STANDARD_PAYLOAD_FLAG_MASK: u8 = 0x10;

pub fn parse_data_submessage(submessage: &[u8], submessage_flags: &u8) -> Result<Data> {
    
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
    let _non_standard_payload_flag =
        submessage_flags & NON_STANDARD_PAYLOAD_FLAG_MASK == NON_STANDARD_PAYLOAD_FLAG_MASK;

    if data_flag && key_flag {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    let extra_flags = deserialize::<u16>(
        submessage,
        &EXTRA_FLAGS_FIRST_INDEX,
        &EXTRA_FLAGS_LAST_INDEX,
        &submessage_endianess,
    )?;
    if extra_flags != 0 {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    let octecs_to_inline_qos = deserialize::<u16>(
        submessage,
        &OCTETS_TO_INLINE_QOS_FIRST_INDEX,
        &OCTETS_TO_INLINE_QOS_LAST_INDEX,
        &submessage_endianess,
    )? as usize;

    let reader_id = deserialize::<EntityId>(
        submessage,
        &READER_ID_FIRST_INDEX,
        &READER_ID_LAST_INDEX,
        &submessage_endianess,
    )?;

    let writer_id = deserialize::<EntityId>(
        submessage,
        &WRITER_ID_FIRST_INDEX,
        &WRITER_ID_LAST_INDEX,
        &submessage_endianess,
    )?;

    let writer_sn: SequenceNumber = deserialize::<SequenceNumberSerialization>(
        submessage,
        &WRITER_SN_FIRST_INDEX,
        &WRITER_SN_LAST_INDEX,
        &submessage_endianess,
    )?
    .into();

    // Octets to data is considered as having the same meaning as octets to inline qos,
    // i.e. counting from the byte after the octets to inline qos field

    let (inline_qos, octets_to_data) = if inline_qos_flag {
        let inline_qos_first_index = OCTETS_TO_INLINE_QOS_LAST_INDEX + octecs_to_inline_qos + 1;
        let (parameter_list, parameter_list_size) = parse_inline_qos_parameter_list(
            submessage,
            &inline_qos_first_index,
            &submessage_endianess,
        )?;
        let octets_to_data = octecs_to_inline_qos + parameter_list_size;
        (Some(parameter_list), octets_to_data)
    } else {
        (None, octecs_to_inline_qos)
    };

    let payload_first_index = OCTETS_TO_INLINE_QOS_LAST_INDEX + octets_to_data + 1;

    let serialized_payload = if data_flag && !key_flag {
        Payload::Data(submessage[payload_first_index..].to_vec())
    } else if !data_flag && key_flag {
        Payload::Key(submessage[payload_first_index..].to_vec())
    } else {
        Payload::None
    };

    Ok(Data {
        reader_id,
        writer_id,
        key_hash: [0;16], /*TODO: key_hash*/
        writer_sn,
        inline_qos,
        serialized_payload,
    })
}

impl Serialize for Data {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut flags : u8 = 1; // Fixed Little Endian endianness

        if self.inline_qos.is_some() {
            flags |= INLINE_QOS_FLAG_MASK;
        }

        match self.serialized_payload {
            Payload::Data(_) => flags |= DATA_FLAG_MASK,
            Payload::Key(_) => flags |= KEY_FLAG_MASK,
            Payload::NonStandard(_) => flags |= NON_STANDARD_PAYLOAD_FLAG_MASK,
            _ => (),
        }

        let payload_size = match &self.serialized_payload {
            Payload::Data(data) | Payload::Key(data) | Payload::NonStandard(data)  => data.len(),
            _ => 0,
        };

        println!("Payload size {:?}", payload_size);

        let octets_to_next_header = (24 + payload_size + 20) as u16; /*TODO: inline_qos_size is fixed at 20 for now */
        const EXTRA_FLAGS : u16 = 0;
        const OCTETS_TO_INLINE_QOS: u16 = 16;

        let writer_sn_msb = ((*self.writer_sn() as u64 & 0xFFFFFFFF00000000) >> 32 ) as i32;
        let writer_sn_lsb = (*self.writer_sn() as u64 & 0x00000000FFFFFFFF) as i32;
        let mut state = serializer.serialize_struct("Data", 5)?;

        state.serialize_field("Header",&(SubmessageKind::Data as u8))?;
        state.serialize_field("Flags",&flags)?;
        state.serialize_field("OctetsToNextHeader",&octets_to_next_header)?;
        state.serialize_field("ExtraFlags",&EXTRA_FLAGS)?;
        state.serialize_field("OctetsToInlineQos",&OCTETS_TO_INLINE_QOS)?;
        state.serialize_field("ReaderId",self.reader_id())?;
        state.serialize_field("WriterId",self.writer_id())?;

        state.serialize_field("WriterSn",&writer_sn_msb)?;
        state.serialize_field("WriterSn",&writer_sn_lsb)?;

        if let Some(inline_qos) = self.inline_qos() {
            state.serialize_field("InlineQoS", inline_qos)?;
        }
        
        // Use the serde bytebuf wrapper to ensure that the data is interpreted as a byte array rather than a sequence
        match self.serialized_payload() {
            Payload::Data(data) | Payload::NonStandard(data) | Payload::Key(data) => state.serialize_field("Serialized Payload Data",  &ByteBuf::from(data.as_slice())),
            Payload::None => Ok(()),
        }?;

        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::InlineQosParameter;
    use crate::types::{ENTITYID_UNKNOWN, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER};

    const DATA_SUBMESSAGE_BIG_ENDIAN: [u8; 60] = [
        0x00, 0x00, 0x00, 0x10, 0x10, 0x12, 0x14, 0x16, 0x26, 0x24, 0x22, 0x20, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x04, 0xD1, 0x00, 0x70, 0x00, 0x10, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
        0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x00, 0x10, 0x00, 0x08, 0x10,
        0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x00, 0x01, 0x00, 0x00, 0x20, 0x30, 0x40, 0x50,
    ];

    #[test]
    fn test_parse_data_submessage_without_inline_qos_or_data() {
        let data = parse_data_submessage(&DATA_SUBMESSAGE_BIG_ENDIAN, &0).unwrap();
        assert_eq!(data.reader_id, EntityId::new([0x10, 0x12, 0x14], 0x16));
        assert_eq!(data.writer_id, EntityId::new([0x26, 0x24, 0x22], 0x20));
        assert_eq!(data.writer_sn, 1233);
        assert_eq!(data.inline_qos, None);
        assert_eq!(data.serialized_payload, Payload::None);
    }

    #[test]
    fn test_parse_data_submessage_with_inline_qos_without_data() {
        let data = parse_data_submessage(&DATA_SUBMESSAGE_BIG_ENDIAN, &2).unwrap();
        assert_eq!(data.reader_id, EntityId::new([0x10, 0x12, 0x14], 0x16));
        assert_eq!(data.writer_id, EntityId::new([0x26, 0x24, 0x22], 0x20));
        assert_eq!(data.writer_sn, 1233);
        assert_eq!(data.serialized_payload, Payload::None);
        let inline_qos = data.inline_qos.unwrap();
        assert_eq!(inline_qos.len(), 1);
        assert_eq!(
            inline_qos[0],
            InlineQosParameter::KeyHash([
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                0x0F, 0x10,
            ])
        );
    }

    #[test]
    fn test_parse_data_submessage_with_inline_qos_and_data() {
        let data = parse_data_submessage(&DATA_SUBMESSAGE_BIG_ENDIAN, &6).unwrap();
        assert_eq!(data.reader_id, EntityId::new([0x10, 0x12, 0x14], 0x16));
        assert_eq!(data.writer_id, EntityId::new([0x26, 0x24, 0x22], 0x20));
        assert_eq!(data.writer_sn, 1233);
        let inline_qos = data.inline_qos.unwrap();
        assert_eq!(inline_qos.len(), 1);
        assert_eq!(
            inline_qos[0],
            InlineQosParameter::KeyHash([
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                0x0F, 0x10,
            ])
        );
        if let Payload::Data(serialized_data) = data.serialized_payload {
            assert_eq!(serialized_data, vec!(0x20, 0x30, 0x40, 0x50,));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_parse_data_submessage_with_inline_qos_and_serialized_key() {
        let data = parse_data_submessage(&DATA_SUBMESSAGE_BIG_ENDIAN, &10).unwrap();
        assert_eq!(data.reader_id, EntityId::new([0x10, 0x12, 0x14], 0x16));
        assert_eq!(data.writer_id, EntityId::new([0x26, 0x24, 0x22], 0x20));
        assert_eq!(data.writer_sn, 1233);
        let inline_qos = data.inline_qos.unwrap();
        assert_eq!(inline_qos.len(), 1);
        assert_eq!(
            inline_qos[0],
            InlineQosParameter::KeyHash([
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
                0x0F, 0x10,
            ])
        );
        if let Payload::Key(serialized_data) = data.serialized_payload {
            assert_eq!(serialized_data, vec!(0x20, 0x30, 0x40, 0x50,));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_parse_data_submessage_without_inline_qos_with_serialized_data() {
        // Parse message considering serialized data and no inline qos
        let data = parse_data_submessage(&DATA_SUBMESSAGE_BIG_ENDIAN, &8).unwrap();
        assert_eq!(data.reader_id, EntityId::new([0x10, 0x12, 0x14], 0x16));
        assert_eq!(data.writer_id, EntityId::new([0x26, 0x24, 0x22], 0x20));
        assert_eq!(data.writer_sn, 1233);
        assert_eq!(data.inline_qos, None);
        if let Payload::Key(serialized_data) = data.serialized_payload {
            assert_eq!(
                serialized_data,
                vec!(
                    0x00, 0x70, 0x00, 0x10, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
                    0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x00, 0x10, 0x00, 0x08, 0x10, 0x11,
                    0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x00, 0x01, 0x00, 0x00, 0x20, 0x30, 0x40,
                    0x50,
                )
            );
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_serialize_data_message() {
        let expected_serialized_data = 
        vec![0x15, 0x07, 0x1c, 0x01, //032 submessageId: SubmessageKind => DATA | flags: SubmessageFlag[8] => N=0|K=0|D=1|Q=1|E=1 Endianess=little && InlineQosFlag && serializedPayload contains data | submessageLength (octetsToNextHeader): ushort => 284
            0x00, 0x00, 0x10, 0x00, //036  [Data Submessage] Flags: extraFlags | octetsToInlineQos: ushort => 16
            0x00, 0x00, 0x00, 0x00, //040  [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, //044  [Data Submessage] EntityId writerId => ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER ([0, 0x01, 0x00], ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY)
            0x00, 0x00, 0x00, 0x00, //048  [Data Submessage] SequenceNumber writerSN 
            0x01, 0x00, 0x00, 0x00, //052  [Data Submessage] SequenceNumber writerSN => 1
            0x70, 0x00, 0x10, 0x00, //056  [Data Submessage: inLineQos] parameterId_1: short => PID_KEY_HASH | length: short => 16
            0x7f, 0x20, 0xf7, 0xd7, //060  [Data Submessage: inLineQos: KEY_HASH] 
            0x00, 0x00, 0x01, 0xbb, //064  [Data Submessage: inLineQos: KEY_HASH] 
            0x00, 0x00, 0x00, 0x01, //068  [Data Submessage: inLineQos: KEY_HASH]  
            0x00, 0x00, 0x01, 0xc1, //072  [Data Submessage: inLineQos: KEY_HASH]  
            0x01, 0x00, 0x00, 0x00, //076  [Data Submessage]  parameterId_1: short => PID_SENTINEL | 0
            0x00, 0x03, 0x00, 0x00, //080  [Data Submessage: SerializedPayload]   representation_identifier: octet[2] => PL_CDR_LE | representation_options: octet[2] => none
            0x15, 0x00, 0x04, 0x00, //084  [Data Submessage: SerializedPayload]   parameterId_1: short => PID_PROTOCOL_VERSION | length: short => 4
            0x02, 0x01, 0x00, 0x00, //088  [Data Submessage: SerializedPayload: PID_PROTOCOL_VERSION]  major: octet => 2 | minor: octet =>1 | padding 
            0x16, 0x00, 0x04, 0x00, //092  [Data Submessage: SerializedPayload]  parameterId_1: short => PID_VENDORID  | length: short => 4
            0x01, 0x02, 0x00, 0x00, //096  [Data Submessage: SerializedPayload: PID_VENDORID] vendorId: octet[2] => 12
            0x31, 0x00, 0x18, 0x00, //100  [Data Submessage: SerializedPayload]  parameterId_1: short =>  PID_DEFAULT_UNICAST_LOCATOR | length: short => 24
            0x01, 0x00, 0x00, 0x00, //104  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
            0xf3, 0x1c, 0x00, 0x00, //108  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
            0x00, 0x00, 0x00, 0x00, //112  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
            0x00, 0x00, 0x00, 0x00, //116  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]  
            0x00, 0x00, 0x00, 0x00, //120  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
            0xc0, 0xa8, 0x02, 0x04, //124  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]
            0x32, 0x00, 0x18, 0x00, //128  [Data Submessage: SerializedPayload] parameterId_1: short => PID_METATRAFFIC_UNICAST_LOCATOR | length: short => 24
            0x01, 0x00, 0x00, 0x00, //132  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
            0xf2, 0x1c, 0x00, 0x00, //136  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
            0x00, 0x00, 0x00, 0x00, //140  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
            0x00, 0x00, 0x00, 0x00, //144  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]  
            0x00, 0x00, 0x00, 0x00, //148  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
            0xc0, 0xa8, 0x02, 0x04, //152  [Data Submessage: SerializedPayload: PID_DEFAULT_UNICAST_LOCATOR]   
            0x02, 0x00, 0x08, 0x00, //156  [Data Submessage: SerializedPayload] parameterId_1: short => PID_PARTICIPANT_LEASE_DURATION | length: short => 8
            0x0b, 0x00, 0x00, 0x00, //160  [Data Submessage: SerializedPayload: PID_PARTICIPANT_LEASE_DURATION] seconds: long => 11 
            0x00, 0x00, 0x00, 0x00, //164  [Data Submessage: SerializedPayload: PID_PARTICIPANT_LEASE_DURATION] fraction: ulong => 0    
            0x50, 0x00, 0x10, 0x00, //168  [Data Submessage: SerializedPayload] parameterId_1: short => PID_PARTICIPANT_GUID | length: short => 16
            0x7f, 0x20, 0xf7, 0xd7, //172  [Data Submessage: SerializedPayload: PID_PARTICIPANT_GUID] 
            0x00, 0x00, 0x01, 0xbb, //176  [Data Submessage: SerializedPayload: PID_PARTICIPANT_GUID]   
            0x00, 0x00, 0x00, 0x01, //180  [Data Submessage: SerializedPayload: PID_PARTICIPANT_GUID]   
            0x00, 0x00, 0x01, 0xc1, //184  [Data Submessage: SerializedPayload: PID_PARTICIPANT_GUID]   
            0x58, 0x00, 0x04, 0x00, //188  [Data Submessage: SerializedPayload] parameterId_1: short => PID_BUILTIN_ENDPOINT_SET | length: short => 4
            0x15, 0x04, 0x00, 0x00, //192  [Data Submessage: SerializedPayload: PID_BUILTIN_ENDPOINT_SET] BuiltinEndpointSet: bitmask => (0100 0001 0101‬) PARTICIPANT_ANNOUNCER && PUBLICATIONS_ANNOUNCER && SUBSCRIPTIONS_ANNOUNCER && PARTICIPANT_MESSAGE_DATA_WRITER
            0x00, 0x80, 0x04, 0x00, //196  [Data Submessage: SerializedPayload] parameterId_1: short => Vendor-specific ParameterId (0x8000) | length: short => 4   
            0x15, 0x00, 0x00, 0x00, //200  [Data Submessage: SerializedPayload: Vendor-specific 0x0]  
            0x07, 0x80, 0x5c, 0x00, //204  [Data Submessage: SerializedPayload] parameterId_1: short => Vendor-specific ParameterId (0x8007) | length: short => 92     
            0x00, 0x00, 0x00, 0x00, //208  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x2f, 0x00, 0x00, 0x00, //212  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x05, 0x00, 0x00, 0x00, //216  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x00, 0x00, 0x00, 0x00, //220  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x50, 0x00, 0x00, 0x00, //224  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x42, 0x00, 0x00, 0x00, //228  [Data Submessage: SerializedPayload: Vendor-specific 0x7]  
            0x44, 0x45, 0x53, 0x4b, //232  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x54, 0x4f, 0x50, 0x2d, //236  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x4f, 0x52, 0x46, 0x44, //240  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x4f, 0x53, 0x35, 0x2f, //244  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x36, 0x2e, 0x31, 0x30, //248  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x2e, 0x32, 0x2f, 0x63, //252  [Data Submessage: SerializedPayload: Vendor-specific 0x7]  
            0x63, 0x36, 0x66, 0x62, //256  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x39, 0x61, 0x62, 0x33, //260  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x36, 0x2f, 0x39, 0x30, //264  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x37, 0x65, 0x66, 0x66, //268  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x30, 0x32, 0x65, 0x33, //272  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x2f, 0x22, 0x78, 0x38, //276  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x36, 0x5f, 0x36, 0x34, //280  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x2e, 0x77, 0x69, 0x6e, //284  [Data Submessage: SerializedPayload: Vendor-specific 0x7]  
            0x2d, 0x76, 0x73, 0x32, //288  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x30, 0x31, 0x35, 0x22, //292  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x2f, 0x00, 0x00, 0x00, //296  [Data Submessage: SerializedPayload: Vendor-specific 0x7]   
            0x25, 0x80, 0x0c, 0x00, //300  [Data Submessage: SerializedPayload] parameterId_1: short => Vendor-specific ParameterId (0x8025) | length: short => 12       
            0xd7, 0xf7, 0x20, 0x7f, //304  [Data Submessage: SerializedPayload: Vendor-specific ParameterId 0x25]   
            0xbb, 0x01, 0x00, 0x00, //308  [Data Submessage: SerializedPayload: Vendor-specific ParameterId 0x25]   
            0x01, 0x00, 0x00, 0x00, //312  [Data Submessage: SerializedPayload: Vendor-specific ParameterId 0x25]  
            0x01, 0x00, 0x00, 0x00, //316  [Data Submessage: SerializedPayload] parameterId_1: short => PID_SENTINEL |  length: short => 0
        ];

        let payload = vec![0x00, 0x03, 0x00, 0x00,
        0x15, 0x00, 0x04, 0x00,
        0x02, 0x01, 0x00, 0x00,
        0x16, 0x00, 0x04, 0x00,
        0x01, 0x02, 0x00, 0x00,
        0x31, 0x00, 0x18, 0x00,
        0x01, 0x00, 0x00, 0x00,
        0xf3, 0x1c, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0xc0, 0xa8, 0x02, 0x04,
        0x32, 0x00, 0x18, 0x00,
        0x01, 0x00, 0x00, 0x00,
        0xf2, 0x1c, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0xc0, 0xa8, 0x02, 0x04,
        0x02, 0x00, 0x08, 0x00,
        0x0b, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x50, 0x00, 0x10, 0x00,
        0x7f, 0x20, 0xf7, 0xd7,
        0x00, 0x00, 0x01, 0xbb,
        0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x01, 0xc1,
        0x58, 0x00, 0x04, 0x00,
        0x15, 0x04, 0x00, 0x00,
        0x00, 0x80, 0x04, 0x00,
        0x15, 0x00, 0x00, 0x00,
        0x07, 0x80, 0x5c, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x2f, 0x00, 0x00, 0x00,
        0x05, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
        0x50, 0x00, 0x00, 0x00,
        0x42, 0x00, 0x00, 0x00,
        0x44, 0x45, 0x53, 0x4b,
        0x54, 0x4f, 0x50, 0x2d,
        0x4f, 0x52, 0x46, 0x44,
        0x4f, 0x53, 0x35, 0x2f,
        0x36, 0x2e, 0x31, 0x30,
        0x2e, 0x32, 0x2f, 0x63,
        0x63, 0x36, 0x66, 0x62,
        0x39, 0x61, 0x62, 0x33,
        0x36, 0x2f, 0x39, 0x30,
        0x37, 0x65, 0x66, 0x66,
        0x30, 0x32, 0x65, 0x33,
        0x2f, 0x22, 0x78, 0x38,
        0x36, 0x5f, 0x36, 0x34,
        0x2e, 0x77, 0x69, 0x6e,
        0x2d, 0x76, 0x73, 0x32,
        0x30, 0x31, 0x35, 0x22,
        0x2f, 0x00, 0x00, 0x00,
        0x25, 0x80, 0x0c, 0x00,
        0xd7, 0xf7, 0x20, 0x7f,
        0xbb, 0x01, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00,];

        let data = Data::new(
            ENTITYID_UNKNOWN, /*reader_id*/
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, /*writer_id*/
            [0;16], /*key_hash*/
            1, /*writer_sn*/
            Some(InlineQosParameterList::new_from_vec(vec![InlineQosParameter::KeyHash([0x7f, 0x20, 0xf7, 0xd7, 0x00, 0x00, 0x01, 0xbb,0x00, 0x00, 0x00, 0x01,0x00, 0x00, 0x01, 0xc1])])), /*inline_qos*/
            Payload::Data(payload) /*serialized_payload*/);
        
        let serialized_data = cdr::ser::serialize_data::<_,_,LittleEndian>(&data, Infinite).unwrap();
        println!("Data: {:x?}", serialized_data);

        assert_eq!(serialized_data, expected_serialized_data);
    }
}
