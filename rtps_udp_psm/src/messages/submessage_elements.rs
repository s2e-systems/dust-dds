use crate::{messages, types};
use rust_rtps_pim::messages::submessages::submessage_elements;

pub type GuidPrefix = types::GuidPrefix;
impl submessage_elements::SubmessageElement for GuidPrefix {}
impl submessage_elements::GuidPrefix for GuidPrefix {
    type GuidPrefix = Self;

    fn value(&self) -> &Self::GuidPrefix {
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
        <Self as rust_rtps_pim::types::ProtocolVersion>::PROTOCOLVERSION_1_0;

    const PROTOCOLVERSION_1_1: Self =
        <Self as rust_rtps_pim::types::ProtocolVersion>::PROTOCOLVERSION_1_1;

    const PROTOCOLVERSION_2_0: Self =
        <Self as rust_rtps_pim::types::ProtocolVersion>::PROTOCOLVERSION_2_0;

    const PROTOCOLVERSION_2_1: Self =
        <Self as rust_rtps_pim::types::ProtocolVersion>::PROTOCOLVERSION_2_1;

    const PROTOCOLVERSION_2_2: Self =
        <Self as rust_rtps_pim::types::ProtocolVersion>::PROTOCOLVERSION_2_2;

    const PROTOCOLVERSION_2_3: Self =
        <Self as rust_rtps_pim::types::ProtocolVersion>::PROTOCOLVERSION_2_3;

    const PROTOCOLVERSION_2_4: Self =
        <Self as rust_rtps_pim::types::ProtocolVersion>::PROTOCOLVERSION_2_4;
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
    pub base: <Self as submessage_elements::SequenceNumberSet>::SequenceNumber,
    pub set: Vec<<Self as submessage_elements::SequenceNumberSet>::SequenceNumber>,
}
impl submessage_elements::SubmessageElement for SequenceNumberSet {}
impl submessage_elements::SequenceNumberSet for SequenceNumberSet {
    type SequenceNumber = types::SequenceNumber;

    fn base(&self) -> &Self::SequenceNumber {
        &self.base
    }

    fn set(&self) -> &[Self::SequenceNumber] {
        &self.set
    }
}

pub type FragmentNumber = messages::types::FragmentNumber;
impl submessage_elements::SubmessageElement for FragmentNumber {}
impl submessage_elements::FragmentNumber for FragmentNumber {
    type FragmentNumber = Self;

    fn value(&self) -> &Self::FragmentNumber {
        self
    }
}

pub struct FragmentNumberSet {
    pub base: <Self as submessage_elements::FragmentNumberSet>::FragmentNumber,
    pub set: Vec<<Self as submessage_elements::FragmentNumberSet>::FragmentNumber>,
}
impl submessage_elements::SubmessageElement for FragmentNumberSet {}
impl submessage_elements::FragmentNumberSet for FragmentNumberSet {
    type FragmentNumber = messages::types::FragmentNumber;

    fn base(&self) -> &Self::FragmentNumber {
        &self.base
    }

    fn set(&self) -> &[Self::FragmentNumber] {
        &self.set
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

pub struct ParameterList {
    pub parameter: <Self as submessage_elements::ParameterList>::ParameterList,
}
impl submessage_elements::SubmessageElement for ParameterList {}
impl submessage_elements::ParameterList for ParameterList {
    type Parameter =
        dyn submessage_elements::Parameter<ParameterId = crate::messages::types::ParameterId>;
    type Item = Box<Self::Parameter>;
    type ParameterList = Vec<Self::Item>;

    fn parameter(&self) -> &Self::ParameterList {
        &self.parameter
    }
}

pub type Count = messages::types::Count;
impl submessage_elements::SubmessageElement for Count {}
impl submessage_elements::Count for Count {
    type Count = Self;

    fn value(&self) -> &Self::Count {
        self
    }
}

pub struct LocatorList(pub Vec<<Self as submessage_elements::LocatorList>::Locator>);
impl submessage_elements::SubmessageElement for LocatorList {}
impl submessage_elements::LocatorList for LocatorList {
    type Locator = types::Locator;

    fn value(&self) -> &[Self::Locator] {
        &self.0
    }
}

pub struct SerializedData<'a>(pub &'a [u8]);
impl<'a> submessage_elements::SubmessageElement for SerializedData<'a> {}
impl<'a> submessage_elements::SerializedData for SerializedData<'a> {
    fn value(&self) -> &[u8] {
        &self.0
    }
}

pub struct SerializedDataFragment<'a>(pub &'a [u8]);
impl<'a> submessage_elements::SubmessageElement for SerializedDataFragment<'a> {}
impl<'a> submessage_elements::SerializedDataFragment for SerializedDataFragment<'a> {
    fn value(&self) -> &[u8] {
        &self.0
    }
}
