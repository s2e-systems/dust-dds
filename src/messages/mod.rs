pub mod types;
mod ack_nack_submessage;
mod data_frag_submessage;
mod data_submessage;
mod gap_submessage;
mod heartbeat_frag_submessage;
mod heartbeat_submessage;
mod info_destination_submessage;
mod info_reply_submessage;
mod info_source_submessage;
mod info_timestamp_submessage;
mod nack_frag_submessage;


use crate::types_primitives::Ushort;
use crate::types::{GuidPrefix, ProtocolVersion, VendorId, };
use crate::types::constants::{PROTOCOL_VERSION_2_4, VENDOR_ID, };
use self::types::{SubmessageKind, SubmessageFlag, ProtocolId, };
use self::types::constants::PROTOCOL_RTPS;
use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsCompose, RtpsParse, EndianessFlag, RtpsSerdesResult, RtpsSerdesError, PrimitiveSerdes, };



// pub use ack_nack_submessage::AckNack;
// pub use data_frag_submessage::DataFrag;
pub use data_submessage::Data;
pub use data_submessage::Payload;
pub use gap_submessage::Gap;
// pub use heartbeat_frag_submessage::HeartbeatFrag;
pub use heartbeat_submessage::Heartbeat;
// pub use info_destination_submessage::InfoDst;
// pub use info_reply_submessage::InfoReply;
// pub use info_source_submessage::InfoSrc;
pub use info_timestamp_submessage::InfoTs;
// pub use nack_frag_submessage::NackFrag;

#[derive(Debug)]
pub enum RtpsMessageError {
    MessageTooSmall,
    InvalidHeader,
    RtpsMajorVersionUnsupported,
    RtpsMinorVersionUnsupported,
    InvalidSubmessageHeader,
    InvalidSubmessage,
    InvalidKeyAndDataFlagCombination,
    IoError(std::io::Error),
    SerdesError(RtpsSerdesError),
    InvalidTypeConversion,
    DeserializationMessageSizeTooSmall,
}

impl From<RtpsSerdesError> for RtpsMessageError {
    fn from(error: RtpsSerdesError) -> Self {
        RtpsMessageError::SerdesError(error)
    }
}

pub type RtpsMessageResult<T> = std::result::Result<T, RtpsMessageError>;

pub const RTPS_MAJOR_VERSION: u8 = 2;
pub const RTPS_MINOR_VERSION: u8 = 4;


#[derive(Debug, PartialEq)]
pub enum RtpsSubmessage {
    // AckNack(AckNack),
    Data(Data),
    // DataFrag(DataFrag),
    Gap(Gap),
    Heartbeat(Heartbeat),
    // HeartbeatFrag(HeartbeatFrag),
    // InfoDst(InfoDst),
    // InfoReply(InfoReply),
    // InfoSrc(InfoSrc),
    InfoTs(InfoTs),
    // NackFrag(NackFrag),
}

impl RtpsCompose for RtpsSubmessage {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        match self {
            RtpsSubmessage::Data(data) => data.compose(writer),
            RtpsSubmessage::Gap(_gap) => Ok(()), //gap.compose(writer)?,
            RtpsSubmessage::Heartbeat(heartbeat) => heartbeat.compose(writer),
            RtpsSubmessage::InfoTs(infots) => infots.compose(writer),
        }
    }
}

impl RtpsParse for RtpsSubmessage {
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
        let submessage_id = SubmessageKind::deserialize(bytes, EndianessFlag::LittleEndian /*irrelevant*/)?;
        match submessage_id {
            SubmessageKind::Data => Ok( RtpsSubmessage::Data(Data::parse(bytes)?) ),
            SubmessageKind::Pad => Err(RtpsSerdesError::InvalidSubmessageHeader),
            SubmessageKind::AckNack => Err(RtpsSerdesError::InvalidSubmessageHeader),
            SubmessageKind::Heartbeat => Ok( RtpsSubmessage::Heartbeat(Heartbeat::parse(bytes)?) ),
            SubmessageKind::Gap => Err(RtpsSerdesError::InvalidSubmessageHeader),
            SubmessageKind::InfoTimestamp => Ok( RtpsSubmessage::InfoTs(InfoTs::parse(bytes)?) ),
            SubmessageKind::InfoSource => Err(RtpsSerdesError::InvalidSubmessageHeader),
            SubmessageKind::InfoReplyIP4 => Err(RtpsSerdesError::InvalidSubmessageHeader),
            SubmessageKind::InfoDestination => Err(RtpsSerdesError::InvalidSubmessageHeader),
            SubmessageKind::InfoReply => Err(RtpsSerdesError::InvalidSubmessageHeader),
            SubmessageKind::NackFrag => Err(RtpsSerdesError::InvalidSubmessageHeader),
            SubmessageKind::HeartbeatFrag => Err(RtpsSerdesError::InvalidSubmessageHeader),
            SubmessageKind::DataFrag => Err(RtpsSerdesError::InvalidSubmessageHeader),
        }   
    }    
}



struct OctetsToNextHeader(u16);

impl RtpsSerialize for OctetsToNextHeader
{
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> {
        writer.write(&PrimitiveSerdes::serialize_u16(self.0, endianness))?;

        Ok(())
    }
}



#[derive(PartialEq, Debug)]
pub struct SubmessageHeader {
    submessage_id: SubmessageKind,
    flags: [SubmessageFlag; 8],
    submessage_length: Ushort,
}

impl SubmessageHeader {
    pub fn submessage_id(&self) -> SubmessageKind {self.submessage_id}
    pub fn flags(&self) -> &[SubmessageFlag; 8] {&self.flags}
    pub fn submessage_length(&self) -> Ushort {self.submessage_length}
}

impl RtpsCompose for SubmessageHeader {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        let endianness = EndianessFlag::from(self.flags[0].is_set());
        self.submessage_id.serialize(writer, endianness)?;
        self.flags.serialize(writer, endianness)?;
        self.submessage_length.serialize(writer, endianness)?;
        Ok(())
    }
}

impl RtpsParse for SubmessageHeader {
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {   
        let submessage_id = SubmessageKind::deserialize(&bytes[0..1], EndianessFlag::LittleEndian /*irrelevant*/)?;
        let flags = <[SubmessageFlag; 8]>::deserialize(&bytes[1..2], EndianessFlag::LittleEndian /*irrelevant*/)?;
        let endianness = EndianessFlag::from(flags[0].is_set());
        let submessage_length = Ushort::deserialize(&bytes[2..4], endianness)?;
        Ok(SubmessageHeader {
            submessage_id, 
            flags,
            submessage_length,
        })
    }
}

pub trait Submessage {
    fn submessage_header(&self) -> SubmessageHeader;
}


#[derive(PartialEq, Debug)]
pub struct Header {
    protocol: ProtocolId,
    version: ProtocolVersion,
    vendor_id: VendorId,
    guid_prefix: GuidPrefix,
}

impl Header {
    pub fn new(guid_prefix: GuidPrefix) -> Self { 
        Self { 
            protocol: PROTOCOL_RTPS, 
            version: PROTOCOL_VERSION_2_4, 
            vendor_id: VENDOR_ID, 
            guid_prefix, 
        } 
    }
    pub fn guid_prefix(&self) -> &GuidPrefix {
        &self.guid_prefix
    }
}



impl RtpsCompose for Header {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        &self.protocol.serialize(writer, EndianessFlag::LittleEndian /*irrelevant*/)?;
        &self.version.serialize(writer, EndianessFlag::LittleEndian /*irrelevant*/)?;
        &self.vendor_id.serialize(writer, EndianessFlag::LittleEndian /*irrelevant*/)?;
        &self.guid_prefix.serialize(writer, EndianessFlag::LittleEndian /*irrelevant*/)?;
        Ok(())
    }
}

impl RtpsParse for Header {
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
        let protocol = ProtocolId::deserialize(&bytes[0..4], EndianessFlag::LittleEndian /*irrelevant*/)?;
        let version = ProtocolVersion::deserialize(&bytes[4..6], EndianessFlag::LittleEndian /*irrelevant*/)?;
        let vendor_id = VendorId::deserialize(&bytes[6..8], EndianessFlag::LittleEndian /*irrelevant*/)?;
        let guid_prefix = GuidPrefix::deserialize(&bytes[8..20], EndianessFlag::LittleEndian /*irrelevant*/)?;
        Ok(Header {protocol, version, vendor_id, guid_prefix})
    }
}

#[derive(PartialEq, Debug)]
pub struct RtpsMessage {
    header: Header,
    submessages: Vec<RtpsSubmessage>,
}

impl RtpsMessage {
    pub fn new(
        header: Header,
        submessages: Vec<RtpsSubmessage>
    ) -> RtpsMessage {
        // TODO: should panic since the stamdard says: 1 to many messages
        // if submessages.is_empty() {
        //     panic!("At least one submessage is required");
        // };
        RtpsMessage {
            header,
            submessages,
        }
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn push(&mut self, submessage: RtpsSubmessage) {
        self.submessages.push(submessage);
    }

    pub fn submessages(&self) -> &Vec<RtpsSubmessage> {
        &self.submessages
    }
}

impl RtpsCompose for RtpsMessage {
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()> {
        &self.header.compose(writer)?;
        for submessage in &self.submessages {
            submessage.compose(writer)?;
        };
        Ok(())
    }
}

impl RtpsParse for RtpsMessage {
    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self> {
        let size = bytes.len();
        let header = Header::parse(bytes)?;

        let mut submessage_start_index: usize = header.octets();
        let mut submessages = Vec::<RtpsSubmessage>::new();

        while submessage_start_index < size {
            let submessage = RtpsSubmessage::parse(&bytes[submessage_start_index..])?;          
                     
            submessage_start_index += submessage.octets();
            submessages.push(submessage);
        };
        Ok(RtpsMessage {
            header,
            submessages, 
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::types::{Time, Count, };
    use crate::types::{SequenceNumber, EntityId, };
    use crate::types_other::{EntityKey, EntityKind, };
    use crate::types::constants::{ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, ENTITYID_UNKNOWN, };
    

    #[test]
    fn test_parse_message_header() {
        let expected = Header {
            protocol: ProtocolId([b'R', b'T', b'P', b'S']),
            version: ProtocolVersion{major: 2, minor: 1},
            vendor_id: VendorId([1, 2]),
            guid_prefix: GuidPrefix([127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1]),
        };
        
        let bytes = [
            0x52, 0x54, 0x50, 0x53, //000 protocol: ProtocolId_t => 'R', 'T', 'P', 'S',
            0x02, 0x01, 0x01, 0x02, //004 version: ProtocolVersion_t => 2.1 | vendorId: VendorId_t => 1,2
            0x7f, 0x20, 0xf7, 0xd7, //008 guidPrefix: GuidPrefix_t => 127, 32, 247, 215
            0x00, 0x00, 0x01, 0xbb, //012 guidPrefix: GuidPrefix_t => 0, 0, 1, 187
            0x00, 0x00, 0x00, 0x01, //016 guidPrefix: GuidPrefix_t => 0, 0, 0, 1
        ];    
        let result = Header::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }


    #[test]
    fn test_parse_submessage_header() {
        let bytes = [0x15_u8, 0b00000001, 20, 0x0];
        let f = SubmessageFlag(false);
        let flags: [SubmessageFlag; 8] = [SubmessageFlag(true), f, f, f, f, f, f, f];
        let expected = SubmessageHeader {
            submessage_id : SubmessageKind::Data, 
            flags,
            submessage_length: Ushort(20),
        };
        let result = SubmessageHeader::parse(&bytes);
    
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn test_compose_submessage_header() {
        let mut result = Vec::new();

        let f = SubmessageFlag(false);
        let t = SubmessageFlag(true);
        let header = SubmessageHeader {
            submessage_id: SubmessageKind::Data,
            flags: [t, t, f, f, f, f, f, f],
            submessage_length: Ushort(16),
        };
        let expected = vec![0x15, 0b00000011, 16, 0x0];
        header.compose(&mut result).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_compose_submessage() {
        
        let submessage = RtpsSubmessage::Data ( Data::new(
            EndianessFlag::LittleEndian, 
            ENTITYID_UNKNOWN, 
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, 
            SequenceNumber(1),
            None, 
            Payload::None
            )
        );

        let expected = vec![
            0x15_u8, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00,  16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
        ];  
  
        let mut writer = Vec::new();
        submessage.compose(&mut writer).unwrap();
        assert_eq!(expected, writer);        
    }

    #[test]
    fn test_parse_submessage() {
        let bytes = vec![
            0x15_u8, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00,  16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
        ];
        let expected = RtpsSubmessage::Data ( Data::new(
            EndianessFlag::LittleEndian, 
            ENTITYID_UNKNOWN, 
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, 
            SequenceNumber(1),
            None, 
            Payload::None
            )
        );
        let result = RtpsSubmessage::parse(&bytes).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_compose_message() {
        let submessage = RtpsSubmessage::Data ( Data::new(
            EndianessFlag::LittleEndian, 
            ENTITYID_UNKNOWN, 
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, 
            SequenceNumber(1),
            None, 
            Payload::None
            )
        );
        let message = RtpsMessage { 
                header : Header {
                    protocol: ProtocolId([b'R', b'T', b'P', b'S']),
                    version: ProtocolVersion{major: 2, minor: 1},
                    vendor_id: VendorId([1, 2]),
                    guid_prefix: GuidPrefix([127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1]),
                },
                submessages : vec![submessage],                
        };

        let expected = vec![
            0x52, 0x54, 0x50, 0x53, //000 protocol: ProtocolId_t => 'R', 'T', 'P', 'S',
            0x02, 0x01, 0x01, 0x02, //004 version: ProtocolVersion_t => 2.1 | vendorId: VendorId_t => 1,2
            0x7f, 0x20, 0xf7, 0xd7, //008 guidPrefix: GuidPrefix_t => 127, 32, 247, 215
            0x00, 0x00, 0x01, 0xbb, //012 guidPrefix: GuidPrefix_t => 0, 0, 1, 187
            0x00, 0x00, 0x00, 0x01, //016 guidPrefix: GuidPrefix_t => 0, 0, 0, 1
            0x15_u8, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00,  16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
        ];  
        let mut writer = Vec::new();
        message.compose(&mut writer).unwrap();
        assert_eq!(expected, writer);        
    }

    #[test]
    fn test_compose_message_three_data_submessages() {
        let test_time = Time::new(1565525425, 269558339);
        let submessage1 = RtpsSubmessage::InfoTs ( InfoTs::new(Some(test_time), EndianessFlag::LittleEndian)
        );
        let submessage2 = RtpsSubmessage::Data ( Data::new(
            EndianessFlag::LittleEndian, 
            ENTITYID_UNKNOWN, 
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, 
            SequenceNumber(1),
            None, 
            Payload::None
            )
        );
        let submessage3 = RtpsSubmessage::Data ( Data::new(
            EndianessFlag::LittleEndian, 
            ENTITYID_UNKNOWN, 
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, 
            SequenceNumber(2),
            None, 
            Payload::None
            )
        );
        let message = RtpsMessage { 
                header : Header {
                    protocol: ProtocolId([b'R', b'T', b'P', b'S']),
                    version: ProtocolVersion{major: 2, minor: 1},
                    vendor_id: VendorId([1, 2]),
                    guid_prefix: GuidPrefix([127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1]),
                },
                submessages : vec![submessage1, submessage2, submessage3],                
        };

        let expected = vec![
            0x52, 0x54, 0x50, 0x53, //000 protocol: ProtocolId_t => 'R', 'T', 'P', 'S',
            0x02, 0x01, 0x01, 0x02, //004 version: ProtocolVersion_t => 2.1 | vendorId: VendorId_t => 1,2
            0x7f, 0x20, 0xf7, 0xd7, //008 guidPrefix: GuidPrefix_t => 127, 32, 247, 215
            0x00, 0x00, 0x01, 0xbb, //012 guidPrefix: GuidPrefix_t => 0, 0, 1, 187
            0x00, 0x00, 0x00, 0x01, //016 guidPrefix: GuidPrefix_t => 0, 0, 0, 1
            0x09, 0x01, 0x08, 0x00,  // [Info Timestamp] Submessgae Header
            0xB1, 0x05, 0x50, 0x5D, // [Info Timestamp]
            0x43, 0x22, 0x11, 0x10, // [Info Timestamp]
            0x15_u8, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00,  16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
            0x15_u8, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00,  16, 0x0, // ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x02, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 2
        ];  
        let mut writer = Vec::new();
        message.compose(&mut writer).unwrap();
        assert_eq!(expected, writer);        
    }

    #[test]
    fn test_parse_message() {
        let submessage = RtpsSubmessage::Data ( Data::new(
            EndianessFlag::LittleEndian, 
            ENTITYID_UNKNOWN, 
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, 
            SequenceNumber(1),
            None, 
            Payload::None
            )
        );
        let expected = RtpsMessage { 
                header : Header {
                    protocol: ProtocolId([b'R', b'T', b'P', b'S']),
                    version: ProtocolVersion{major: 2, minor: 1},
                    vendor_id: VendorId([1, 2]),
                    guid_prefix: GuidPrefix([127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1]),
                },
                submessages : vec![submessage],                
        };

        let bytes = [
            0x52, 0x54, 0x50, 0x53, //000 protocol: ProtocolId_t => 'R', 'T', 'P', 'S',
            0x02, 0x01, 0x01, 0x02, //004 version: ProtocolVersion_t => 2.1 | vendorId: VendorId_t => 1,2
            0x7f, 0x20, 0xf7, 0xd7, //008 guidPrefix: GuidPrefix_t => 127, 32, 247, 215
            0x00, 0x00, 0x01, 0xbb, //012 guidPrefix: GuidPrefix_t => 0, 0, 1, 187
            0x00, 0x00, 0x00, 0x01, //016 guidPrefix: GuidPrefix_t => 0, 0, 0, 1
            0x15, 0b00000001, 20, 0x0, // Submessgae Header
            0x00, 0x00,  16, 0x0,   // [Data Submessage] ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
        ];    
        let result = RtpsMessage::parse(&bytes).unwrap();
        assert_eq!(expected, result);        


        let submessage1 = RtpsSubmessage::Data ( Data::new(
            EndianessFlag::LittleEndian, 
            ENTITYID_UNKNOWN, 
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, 
            SequenceNumber(1),
            None, 
            Payload::None
            )
        );
        let reader_id = EntityId::new(EntityKey([0x10, 0x12, 0x14]), EntityKind::UserDefinedReaderWithKey);
        let writer_id = EntityId::new(EntityKey([0x26, 0x24, 0x22]), EntityKind::UserDefinedWriterWithKey);
        let first_sn = SequenceNumber(1233);
        let last_sn = SequenceNumber(1237);
        let count = Count(8);
        let final_flag = true;
        let liveliness_flag = false;

        let submessage2 = RtpsSubmessage::Heartbeat ( Heartbeat::new(
            reader_id, writer_id, first_sn, last_sn, count, final_flag, liveliness_flag, EndianessFlag::BigEndian)
        );
        let expected = RtpsMessage { 
                header : Header {
                    protocol: ProtocolId([b'R', b'T', b'P', b'S']),
                    version: ProtocolVersion{major: 2, minor: 1},
                    vendor_id: VendorId([1, 2]),
                    guid_prefix: GuidPrefix([127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1]),
                },
                submessages : vec![submessage1, submessage2],                
        };
        let bytes = [
            0x52, 0x54, 0x50, 0x53, // protocol: ProtocolId_t => 'R', 'T', 'P', 'S',
            0x02, 0x01, 0x01, 0x02, // version: ProtocolVersion_t => 2.1 | vendorId: VendorId_t => 1,2
            0x7f, 0x20, 0xf7, 0xd7, // guidPrefix: GuidPrefix_t => 127, 32, 247, 215
            0x00, 0x00, 0x01, 0xbb, // guidPrefix: GuidPrefix_t => 0, 0, 1, 187
            0x00, 0x00, 0x00, 0x01, // guidPrefix: GuidPrefix_t => 0, 0, 0, 1
            0x15, 0b00000001, 20, 0x0, // Submessgae Header => Data
            0x00, 0x00,  16, 0x0,   // [Data Submessage] ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
            0x07, 0b00000010, 0x00, 28, // Submessage Header => Heartbeat
            0x10, 0x12, 0x14, 0x04, // [Heartbeat Submessage] Reader ID
            0x26, 0x24, 0x22, 0x02, // [Heartbeat Submessage] Writer ID
            0x00, 0x00, 0x00, 0x00, // [Heartbeat Submessage] First Sequence Number
            0x00, 0x00, 0x04, 0xD1, // [Heartbeat Submessage] First Sequence Number
            0x00, 0x00, 0x00, 0x00, // [Heartbeat Submessage] Last Sequence Number
            0x00, 0x00, 0x04, 0xD5, // [Heartbeat Submessage] Last Sequence Number
            0x00, 0x00, 0x00, 0x08, // [Heartbeat Submessage] Count
        ];
        let result = RtpsMessage::parse(&bytes).unwrap();
        assert_eq!(expected, result);  
    }


    #[test]
    fn test_parse_message_two_data_submessages() {
        let test_time = Time::new(1565525425, 269558339);
        let submessage1 = RtpsSubmessage::InfoTs ( InfoTs::new(Some(test_time), EndianessFlag::LittleEndian)
        );
        let submessage2 = RtpsSubmessage::Data ( Data::new(
            EndianessFlag::LittleEndian, 
            ENTITYID_UNKNOWN, 
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, 
            SequenceNumber(1),
            None, 
            Payload::None
            )
        );
        let submessage3 = RtpsSubmessage::Data ( Data::new(
            EndianessFlag::LittleEndian, 
            ENTITYID_UNKNOWN, 
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, 
            SequenceNumber(2),
            None, 
            Payload::None
            )
        );
        let expected = RtpsMessage { 
                header : Header {
                    protocol: ProtocolId([b'R', b'T', b'P', b'S']),
                    version: ProtocolVersion{major: 2, minor: 1},
                    vendor_id: VendorId([1, 2]),
                    guid_prefix: GuidPrefix([127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1]),
                },
                submessages : vec![submessage1, submessage2, submessage3],                
        };

        let bytes = [
            0x52, 0x54, 0x50, 0x53, // protocol: ProtocolId_t => 'R', 'T', 'P', 'S',
            0x02, 0x01, 0x01, 0x02, // version: ProtocolVersion_t => 2.1 | vendorId: VendorId_t => 1,2
            0x7f, 0x20, 0xf7, 0xd7, // guidPrefix: GuidPrefix_t => 127, 32, 247, 215
            0x00, 0x00, 0x01, 0xbb, // guidPrefix: GuidPrefix_t => 0, 0, 1, 187
            0x00, 0x00, 0x00, 0x01, // guidPrefix: GuidPrefix_t => 0, 0, 0, 1
            0x09, 0x01, 0x08, 0x00,  // [Info Timestamp] Submessgae Header
            0xB1, 0x05, 0x50, 0x5D, // [Info Timestamp]
            0x43, 0x22, 0x11, 0x10, // [Info Timestamp]
            0x15, 0b00000001, 20, 0x0, // [Data Submessage] Submessgae Header
            0x00, 0x00,  16, 0x0,   // [Data Submessage] ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x01, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
            0x15, 0b00000001, 20, 0x0, // [Data Submessage] Submessgae Header
            0x00, 0x00,  16, 0x0,   // [Data Submessage] ExtraFlags, octetsToInlineQos (liitle indian)
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] EntityId readerId => ENTITYID_UNKNOWN
            0x00, 0x01, 0x00, 0xc2, // [Data Submessage] EntityId writerId
            0x00, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN
            0x02, 0x00, 0x00, 0x00, // [Data Submessage] SequenceNumber writerSN => 1
        ];    
        let result = RtpsMessage::parse(&bytes).unwrap();
        assert_eq!(expected, result);  
    }
}


