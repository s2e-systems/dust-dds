use byteorder::ReadBytesExt;

use crate::implementation::{
    data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
    rtps::types::{Count, EntityId, GuidPrefix, ProtocolVersion, SequenceNumber, VendorId},
    rtps_udp_psm::mapping_traits::MappingReadByteOrdered,
};

use super::{
    submessage_elements::{FragmentNumberSet, LocatorList, ParameterList, SequenceNumberSet},
    types::{FragmentNumber, SerializedPayload, SubmessageFlag, Time, ULong, UShort, TIME_INVALID},
};

#[derive(Debug, PartialEq, Eq)]
pub struct AckNackSubmessageRead<'a> {
    data: &'a [u8],
}
impl Endianness for AckNackSubmessageRead<'_> {
    fn endianness(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }
}

impl<'a> AckNackSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn endianness_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }

    pub fn final_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0010) != 0
    }

    pub fn reader_id(&self) -> EntityId {
        self.mapping_read(&self.data[4..])
    }

    pub fn writer_id(&self) -> EntityId {
        self.mapping_read(&self.data[8..])
    }

    pub fn reader_sn_state(&self) -> SequenceNumberSet {
        self.mapping_read(&self.data[12..])
    }

    pub fn count(&self) -> Count {
        self.mapping_read(&self.data[self.data.len() - 4..])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AckNackSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub reader_sn_state: SequenceNumberSet,
    pub count: Count,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DataSubmessageRead<'a> {
    data: &'a [u8],
}

pub trait Endianness {
    fn endianness(&self) -> bool;
}

trait MappingRead: Endianness {
    fn mapping_read<'de, T: MappingReadByteOrdered<'de> + 'de>(&self, mut data: &'de [u8]) -> T {
        if self.endianness() {
            T::mapping_read_byte_ordered::<byteorder::LittleEndian>(&mut data).unwrap()
        } else {
            T::mapping_read_byte_ordered::<byteorder::BigEndian>(&mut data).unwrap()
        }
    }
}

impl<T: Endianness> MappingRead for T {}

impl Endianness for DataSubmessageRead<'_> {
    fn endianness(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }
}

impl<'a> DataSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    fn octets_to_inline_qos(&self) -> usize {
        (&self.data[6..])
            .read_u16::<byteorder::LittleEndian>()
            .unwrap() as usize
    }

    fn inline_qos_len(&self) -> usize {
        let mut parameter_list_buf = &self.data[8 + self.octets_to_inline_qos()..];
        let parameter_list_buf_length = parameter_list_buf.len();

        if self.inline_qos_flag() {
            loop {
                let pid = parameter_list_buf
                    .read_u16::<byteorder::LittleEndian>()
                    .expect("pid read failed");
                let length = parameter_list_buf
                    .read_i16::<byteorder::LittleEndian>()
                    .expect("length read failed");
                if pid == PID_SENTINEL {
                    break;
                } else {
                    (_, parameter_list_buf) = parameter_list_buf.split_at(length as usize);
                }
            }
            parameter_list_buf_length - parameter_list_buf.len()
        } else {
            0
        }
    }

    pub fn endianness_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }

    pub fn inline_qos_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0010) != 0
    }

    pub fn data_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0100) != 0
    }

    pub fn key_flag(&self) -> bool {
        (self.data[1] & 0b_0000_1000) != 0
    }

    pub fn non_standard_payload_flag(&self) -> bool {
        (self.data[1] & 0b_0001_0000) != 0
    }

    pub fn reader_id(&self) -> EntityId {
        self.mapping_read(&self.data[8..])
    }

    pub fn writer_id(&self) -> EntityId {
        self.mapping_read(&self.data[12..])
    }

    pub fn writer_sn(&self) -> SequenceNumber {
        self.mapping_read(&self.data[16..])
    }

    pub fn inline_qos(&self) -> ParameterList {
        if self.inline_qos_flag() {
            let mut buf = &self.data[self.octets_to_inline_qos() + 8..];
            match self.endianness_flag() {
                true => {
                    ParameterList::mapping_read_byte_ordered::<byteorder::LittleEndian>(&mut buf)
                        .expect("RtpsParameterList failed LE")
                }
                false => ParameterList::mapping_read_byte_ordered::<byteorder::BigEndian>(&mut buf)
                    .expect("RtpsParameterList failed BE"),
            }
        } else {
            ParameterList::empty()
        }
    }

    pub fn serialized_payload(&self) -> SerializedPayload<'a> {
        SerializedPayload::new(
            &self.data[8 + self.octets_to_inline_qos() + self.inline_qos_len()..],
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DataSubmessageWrite<'a> {
    pub endianness_flag: SubmessageFlag,
    pub inline_qos_flag: SubmessageFlag,
    pub data_flag: SubmessageFlag,
    pub key_flag: SubmessageFlag,
    pub non_standard_payload_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub inline_qos: &'a ParameterList,
    pub serialized_payload: SerializedPayload<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DataFragSubmessageRead<'a> {
    pub endianness_flag: SubmessageFlag,
    pub inline_qos_flag: SubmessageFlag,
    pub non_standard_payload_flag: SubmessageFlag,
    pub key_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub fragment_starting_num: FragmentNumber,
    pub fragments_in_submessage: UShort,
    pub data_size: ULong,
    pub fragment_size: UShort,
    pub inline_qos: &'a [u8],
    pub serialized_payload: SerializedPayload<'a>,
}

impl<'a> DataFragSubmessageRead<'a> {
    pub fn inline_qos(&self) -> ParameterList {
        if self.inline_qos_flag {
            let mut buf = self.inline_qos;
            match self.endianness_flag {
                true => {
                    ParameterList::mapping_read_byte_ordered::<byteorder::LittleEndian>(&mut buf)
                        .expect("RtpsParameterList failed LE")
                }
                false => ParameterList::mapping_read_byte_ordered::<byteorder::BigEndian>(&mut buf)
                    .expect("RtpsParameterList failed BE"),
            }
        } else {
            ParameterList::empty()
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DataFragSubmessageWrite<'a> {
    pub endianness_flag: SubmessageFlag,
    pub inline_qos_flag: SubmessageFlag,
    pub non_standard_payload_flag: SubmessageFlag,
    pub key_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub fragment_starting_num: FragmentNumber,
    pub fragments_in_submessage: UShort,
    pub data_size: ULong,
    pub fragment_size: UShort,
    pub inline_qos: &'a ParameterList,
    pub serialized_payload: SerializedPayload<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GapSubmessageRead<'a> {
    data: &'a [u8],
}
impl Endianness for GapSubmessageRead<'_> {
    fn endianness(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }
}

impl<'a> GapSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn endianness_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }

    pub fn reader_id(&self) -> EntityId {
        self.mapping_read(&self.data[4..])
    }

    pub fn writer_id(&self) -> EntityId {
        self.mapping_read(&self.data[8..])
    }

    pub fn gap_start(&self) -> SequenceNumber {
        self.mapping_read(&self.data[12..])
    }

    pub fn gap_list(&self) -> SequenceNumberSet {
        self.mapping_read(&self.data[20..])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct GapSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub gap_start: SequenceNumber,
    pub gap_list: SequenceNumberSet,
}

#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatSubmessageRead<'a> {
    data: &'a [u8],
}

impl Endianness for HeartbeatSubmessageRead<'_> {
    fn endianness(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }
}

impl<'a> HeartbeatSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn endianness_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }

    pub fn final_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0010) != 0
    }

    pub fn liveliness_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0100) != 0
    }

    pub fn reader_id(&self) -> EntityId {
        self.mapping_read(&self.data[4..])
    }

    pub fn writer_id(&self) -> EntityId {
        self.mapping_read(&self.data[8..])
    }

    pub fn first_sn(&self) -> SequenceNumber {
        self.mapping_read(&self.data[12..])
    }

    pub fn last_sn(&self) -> SequenceNumber {
        self.mapping_read(&self.data[20..])
    }

    pub fn count(&self) -> Count {
        self.mapping_read(&self.data[28..])
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub liveliness_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub first_sn: SequenceNumber,
    pub last_sn: SequenceNumber,
    pub count: Count,
}

#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatFragSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub last_fragment_num: FragmentNumber,
    pub count: Count,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoDestinationSubmessageRead<'a> {
    data: &'a [u8],
}

impl Endianness for InfoDestinationSubmessageRead<'_> {
    fn endianness(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }
}

impl<'a> InfoDestinationSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn endianness_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }

    pub fn guid_prefix(&self) -> GuidPrefix {
        self.mapping_read(&self.data[4..])
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct InfoDestinationSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub guid_prefix: GuidPrefix,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoReplySubmessageRead<'a> {
    data: &'a [u8],
}

impl Endianness for InfoReplySubmessageRead<'_> {
    fn endianness(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }
}

impl<'a> InfoReplySubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn endianness_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }

    pub fn multicast_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0010) != 0
    }

    pub fn unicast_locator_list(&self) -> LocatorList {
        self.mapping_read(&self.data[4..])
    }

    pub fn multicast_locator_list(&self) -> LocatorList {
        if self.multicast_flag() {
            let num_locators: u32 = self.mapping_read(&self.data[4..]);
            let octets_to_multicat_loctor_list = num_locators as usize * 24 + 8;
            self.mapping_read(&self.data[octets_to_multicat_loctor_list..])
        } else {
            LocatorList::new(vec![])
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoReplySubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub multicast_flag: SubmessageFlag,
    pub unicast_locator_list: LocatorList,
    pub multicast_locator_list: LocatorList,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoSourceSubmessageRead<'a> {
    data: &'a [u8],
}

impl Endianness for InfoSourceSubmessageRead<'_> {
    fn endianness(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }
}

impl<'a> InfoSourceSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn endianness_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }

    pub fn protocol_version(&self) -> ProtocolVersion {
        self.mapping_read(&self.data[8..])
    }

    pub fn vendor_id(&self) -> VendorId {
        self.mapping_read(&self.data[10..])
    }

    pub fn guid_prefix(&self) -> GuidPrefix {
        self.mapping_read(&self.data[12..])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoSourceSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub protocol_version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoTimestampSubmessageRead<'a> {
    data: &'a[u8],
}

impl Endianness for InfoTimestampSubmessageRead<'_> {
    fn endianness(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }
}

impl<'a> InfoTimestampSubmessageRead<'a> {
    pub fn new(data: &'a[u8]) -> Self { Self { data } }

    pub fn endianness_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }

    pub fn invalidate_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0010) != 0
    }

    pub fn timestamp(&self) -> Time {
        if self.invalidate_flag() {
            TIME_INVALID
        } else {
            self.mapping_read(&self.data[4..])
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoTimestampSubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub invalidate_flag: SubmessageFlag,
    pub timestamp: Time,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NackFragSubmessage {
    pub endianness_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub fragment_number_state: FragmentNumberSet,
    pub count: Count,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PadSubmessage {
    pub endianness_flag: SubmessageFlag,
}
