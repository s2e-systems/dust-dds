use crate::types;
use rust_rtps_pim::messages::submessages::submessage_elements;

pub type Short = types::Short;
impl submessage_elements::SubmessageElement for Short {}
impl submessage_elements::Short for Short {
    fn value(&self) -> &i16 {
        self
    }
}

pub type UShort = types::UShort;
impl submessage_elements::SubmessageElement for UShort {}
impl submessage_elements::UShort for UShort {
    fn value(&self) -> &u16 {
        self
    }
}

pub type EntityId = types::EntityId;
impl submessage_elements::SubmessageElement for EntityId {}
impl submessage_elements::EntityId for EntityId {
    type EntityId = Self;

    fn value(&self) -> &Self::EntityId {
        self
    }
}

pub type GuidPrefix = types::GuidPrefix;
impl submessage_elements::SubmessageElement for GuidPrefix {}
impl submessage_elements::GuidPrefix for GuidPrefix {
    type GuidPrefix = Self;

    fn value(&self) -> &Self::GuidPrefix {
        self
    }
}

pub type VendorId = types::VendorId;
impl submessage_elements::SubmessageElement for VendorId {}
impl submessage_elements::VendorId for VendorId {
    type VendorId = Self;

    fn value(&self) -> &Self::VendorId {
        self
    }

    const VENDORID_UNKNOWN: Self = <Self as rust_rtps_pim::types::VendorId>::VENDOR_ID_UNKNOWN;
}

pub type ProtocolVersion = types::ProtocolVersion;
impl submessage_elements::SubmessageElement for ProtocolVersion {}
impl submessage_elements::ProtocolVersion for ProtocolVersion {
    type ProtocolVersion = Self;

    fn value(&self) -> &Self::ProtocolVersion {
        self
    }

    const PROTOCOLVERSION_1_0: Self =
        <Self as rust_rtps_pim::types::ProtocolVersion>::PROTOCOL_VERSION_1_0;

    const PROTOCOLVERSION_1_1: Self =
        <Self as rust_rtps_pim::types::ProtocolVersion>::PROTOCOL_VERSION_1_1;

    const PROTOCOLVERSION_2_0: Self =
        <Self as rust_rtps_pim::types::ProtocolVersion>::PROTOCOL_VERSION_2_0;

    const PROTOCOLVERSION_2_1: Self =
        <Self as rust_rtps_pim::types::ProtocolVersion>::PROTOCOL_VERSION_2_1;

    const PROTOCOLVERSION_2_2: Self =
        <Self as rust_rtps_pim::types::ProtocolVersion>::PROTOCOL_VERSION_2_2;

    const PROTOCOLVERSION_2_3: Self =
        <Self as rust_rtps_pim::types::ProtocolVersion>::PROTOCOL_VERSION_2_3;

    const PROTOCOLVERSION_2_4: Self =
        <Self as rust_rtps_pim::types::ProtocolVersion>::PROTOCOL_VERSION_2_4;
}

pub type SequenceNumber = types::SequenceNumber;
impl submessage_elements::SubmessageElement for SequenceNumber {}
impl submessage_elements::SequenceNumber for SequenceNumber {
    type SequenceNumber = Self;

    fn value(&self) -> &Self::SequenceNumber {
        self
    }

    const SEQUENCENUMBER_UNKNOWN: Self =
        <Self as rust_rtps_pim::types::SequenceNumber>::SEQUENCE_NUMBER_UNKNOWN;
}

pub struct SequenceNumberSet {
    bitmap_base: types::SequenceNumber,
    bitmap: [types::Long; 8],
}
impl submessage_elements::SubmessageElement for SequenceNumberSet {}
impl submessage_elements::SequenceNumberSet for SequenceNumberSet {
    type SequenceNumber = types::SequenceNumber;
    type SequenceNumberList = Vec<Self::SequenceNumber>;

    fn base(&self) -> &Self::SequenceNumber {
        &self.bitmap_base
    }

    fn set(&self) -> &Self::SequenceNumberList {
        todo!()
    }
}

pub type Timestamp = crate::messages::types::Time;
impl submessage_elements::SubmessageElement for Timestamp {}
impl submessage_elements::Timestamp for Timestamp {
    type Time = Self;

    fn value(&self) -> &Self::Time {
        self
    }

    const TIME_ZERO: Self = <Self as rust_rtps_pim::messages::types::Time>::TIME_ZERO;

    const TIME_INVALID: Self = <Self as rust_rtps_pim::messages::types::Time>::TIME_INVALID;

    const TIME_INFINITE: Self = <Self as rust_rtps_pim::messages::types::Time>::TIME_INFINITE;
}
