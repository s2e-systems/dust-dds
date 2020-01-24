use std::convert::TryInto;
use num_derive::FromPrimitive;

use std::collections::HashMap;

use crate::types::{VendorId, ProtocolVersion, Locator, LocatorList, Time, GUID, BuiltInEndPointSet};
use crate::parser::helpers::{deserialize, EndianessFlag};

#[derive(Debug)]
pub enum SpdpErrorMessage {
    DataMessageTooSmall,
    InvalidEndianess,
    ParameterLengthTooSmall,
    InvalidParameterLength,
    ParameterContentConversionFailed,
    CdrError(cdr::Error),
}

impl From<cdr::Error> for SpdpErrorMessage {
    fn from(error: cdr::Error) -> Self {
        SpdpErrorMessage::CdrError(error)
    }
}

type Result<T> = std::result::Result< T, SpdpErrorMessage>;

#[derive(FromPrimitive)]
pub enum SpdpParameterId {
    Pad = 0x0000, //N/A
    Sentinel = 0x0001, //N/A
    UserData = 0x002c, //UserDataQosPolicy
    TopicName = 0x0005, //string<256>
    TypeName = 0x0007, //string<256>
    GroupData = 0x002d, //GroupDataQosPolicy
    TopicData = 0x002e, //TopicDataQosPolicy
    Durability = 0x001d, //DurabilityQosPolicy
    DurabilityService = 0x001e, //DurabilityServiceQosPolicy
    Deadline = 0x0023, //DeadlineQosPolicy
    LatencyBudget = 0x0027, //LatencyBudgetQosPolicy
    Liveliness = 0x001b, //LivelinessQosPolicy
    Reliability = 0x001a, //ReliabilityQosPolicy
    Lifespan = 0x002b, //LifespanQosPolicy
    DestinationOrder = 0x0025, //DestinationOrderQosPolicy
    History = 0x0040, //HistoryQosPolicy
    ResourceLimits = 0x0041, //ResourceLimitsQosPolicy
    Ownership = 0x001f, //OwnershipQosPolicy
    OwnershipStrength = 0x0006, //OwnershipStrengthQosPolicy
    Presentation = 0x0021, //PresentationQosPolicy
    Partition = 0x0029, //PartitionQosPolicy
    TimeBasedFilter = 0x0004, //TimeBasedFilterQosPolicy
    TransportPriority = 0x0049, //TransportPriorityQoSPolicy
    DomainId = 0x000f, //DomainId_t
    DomainTag = 0x4014, //string<256>
    ProtocolVersion = 0x0015, //ProtocolVersion_t
    VendorId = 0x0016, //VendorId_t
    UnicastLocator = 0x002f, //Locator_t
    MulticastLocator = 0x0030, //Locator_t
    DefaultUnicastLocator = 0x0031, //Locator_t
    DefaultMulticastLocator = 0x0048, //Locator_t
    MetatrafficUnicastLocator = 0x0032, //Locator_t
    MetatrafficMulticastLocator = 0x0033, //Locator_t
    ExpectsInlineQoS = 0x0043, //boolean
    ParticipantManualLivelinessCount = 0x0034, //Count_t
    ParticipantLeaseDuration = 0x0002, //Duration_t
    ContentFilterProperty = 0x0035, //ContentFilterProperty_t
    ParticipantGuid = 0x0050, //GUID_t
    GroupGuid =  0x0052, //GUID_t
    BuiltinEndpointSet = 0x0058, //BuiltinEndpointSet_t
    BuiltinEndpointQoS = 0x0077, //BuiltinEndpointQos_t
    PropertyList = 0x0059, //sequence<Property_t>
    TypeMaxSizeSerialized = 0x0060, //long
    EntityName = 0x0062, //EntityName_t
    EndpointGuid = 0x005a, //GUID_t
}

#[derive(PartialEq, Debug)]
pub enum SpdpParameter {
    ProtocolVersion(ProtocolVersion),
    VendorId(VendorId),
    UnicastLocator(Locator),
    MulticastLocator(Locator),
    DefaultUnicastLocator(Locator),
    DefaultMulticastLocator(Locator),
    MetatrafficUnicastLocator(Locator),
    MetatrafficMulticastLocator(Locator),
    ParticipantLeaseDuration(Time), //TODO: Replace time type by duration type
    ParticipantGuid(GUID),
    BuiltinEndpointSet(BuiltInEndPointSet),
}

pub type SpdpParameterList = HashMap<SpdpParameterId, SpdpParameter>;

// This function is very much similar to parse_inline_qos_parameter_list in the parser::helpers modules. Later some re-use could be considered
pub fn parse_spdp_parameter_list(data: &[u8]) -> Result<SpdpParameterList>{
    const MINIMUM_DATA_LENGTH: usize = 12; // If a message has data it must have the encoding information, options and at least one parameter with minimum 4 length
    const MINIMUM_PARAMETER_VALUE_LENGTH: usize = 4;
    const FIRST_PARAMETER_LENGTH_OFFSET: usize = 4;

    // This offsets are from the beginning of each parameter ID represented in the list with PARAMETER_ID_FIRST_OFFSET = 0
    const PARAMETER_ID_OFFSET: usize = 1;
    const LENGTH_FIRST_OFFSET: usize = 2;
    const LENGTH_LAST_OFFSET: usize = 3;
    const VALUE_FIRST_OFFSET: usize = 4;

    if data.len() < MINIMUM_DATA_LENGTH {
        return Err(SpdpErrorMessage::DataMessageTooSmall);
    }

    let endianess = if data[0] == 0x00 && data[1] == 0x02 {
        EndianessFlag::BigEndian
    } else if data[0] == 0x00 && data[1] == 0x03 {
        EndianessFlag::LittleEndian
    } else {
        return Err(SpdpErrorMessage::InvalidEndianess);
    };

    let mut parameter_id_first_index = FIRST_PARAMETER_LENGTH_OFFSET;

    let mut parameter_list = Vec::new();

    loop {
        let parameter_id_last_index = parameter_id_first_index + PARAMETER_ID_OFFSET;
        let length_first_index = parameter_id_first_index + LENGTH_FIRST_OFFSET;
        let length_last_index = parameter_id_first_index + LENGTH_LAST_OFFSET;

        let value_first_index = parameter_id_first_index + VALUE_FIRST_OFFSET;
        let value_last_index;

        let parameter_id = deserialize::<u16>(data, &parameter_id_first_index, &parameter_id_last_index, &endianess).unwrap(); //TODO: Remove unwrap
        if parameter_id == SpdpParameterId::Sentinel as u16{
            break;
        }

        let length = deserialize::<u16>(data, &length_first_index, &length_last_index, &endianess).unwrap() as usize; //TODO: Remove unwrap
        if length < MINIMUM_PARAMETER_VALUE_LENGTH {
            return Err(SpdpErrorMessage::ParameterLengthTooSmall);
        }

        value_last_index = value_first_index + length - 1;
        if value_last_index >= data.len() {
            return Err(SpdpErrorMessage::InvalidParameterLength);
        }

        let value = match num::FromPrimitive::from_u16(parameter_id) {
            Some(SpdpParameterId::ProtocolVersion) => Some(SpdpParameter::ProtocolVersion(deserialize::<ProtocolVersion>(&data, &value_first_index, &value_last_index, &endianess).unwrap())),
            Some(SpdpParameterId::VendorId) => Some(SpdpParameter::VendorId(deserialize::<VendorId>(&data, &value_first_index, &value_last_index, &endianess).unwrap())),
            Some(SpdpParameterId::UnicastLocator) => Some(SpdpParameter::UnicastLocator(deserialize::<Locator>(&data, &value_first_index, &value_last_index, &endianess).unwrap())),
            Some(SpdpParameterId::MulticastLocator) => Some(SpdpParameter::MulticastLocator(deserialize::<Locator>(&data, &value_first_index, &value_last_index, &endianess).unwrap())),
            Some(SpdpParameterId::DefaultUnicastLocator) => Some(SpdpParameter::DefaultUnicastLocator(deserialize::<Locator>(&data, &value_first_index, &value_last_index, &endianess).unwrap())),
            Some(SpdpParameterId::DefaultMulticastLocator) => Some(SpdpParameter::DefaultMulticastLocator(deserialize::<Locator>(&data, &value_first_index, &value_last_index, &endianess).unwrap())),
            Some(SpdpParameterId::MetatrafficUnicastLocator) => Some(SpdpParameter::MetatrafficUnicastLocator(deserialize::<Locator>(&data, &value_first_index, &value_last_index, &endianess).unwrap())),
            Some(SpdpParameterId::MetatrafficMulticastLocator) => Some(SpdpParameter::MetatrafficMulticastLocator(deserialize::<Locator>(&data, &value_first_index, &value_last_index, &endianess).unwrap())),
            Some(SpdpParameterId::ParticipantLeaseDuration) => Some(SpdpParameter::ParticipantLeaseDuration(deserialize::<Time>(&data, &value_first_index, &value_last_index, &endianess).unwrap())),
            Some(SpdpParameterId::ParticipantGuid) => Some(SpdpParameter::ParticipantGuid(deserialize::<GUID>(&data, &value_first_index, &value_last_index, &endianess).unwrap())),
            Some(SpdpParameterId::BuiltinEndpointSet) => Some(SpdpParameter::BuiltinEndpointSet(deserialize::<BuiltInEndPointSet>(&data, &value_first_index, &value_last_index, &endianess).unwrap())),
            // Some(InlineQosPid::StatusInfo) => Some(InlineQosParameter::StatusInfo(submessage[value_first_index..=value_last_index].try_into().map_err(|_| ErrorMessage::InvalidSubmessageHeader)?)),
            _ => None,
        };

        if let Some(parameter) = value {
            parameter_list.push(parameter);
        } else {
        }

        parameter_id_first_index = value_last_index + 1;
    }

    Ok(parameter_list)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EntityId, ENTITY_KIND_BUILT_IN_PARTICIPANT};

    #[test]
    fn test_parse_example_spdp_data() {
        let data = [0, 3, 0, 0, 21, 0, 4, 0, 2, 1, 0, 0, 22, 0, 4, 0, 1, 2, 0, 0, 49, 0, 24, 0, 1, 0, 0, 0, 243, 28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 168, 1, 160, 72, 0, 24, 0, 1, 0, 0, 0, 233, 28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1, 50, 0, 24, 0, 1, 0, 0, 0, 242, 28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 168, 1, 160, 51, 0, 24, 0, 1, 0, 0, 0, 232, 28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1, 2, 0, 8, 0, 11, 0, 0, 0, 0, 0, 0, 0, 80, 0, 16, 0, 6, 85, 244, 87, 0, 0, 0, 4, 0, 0, 0, 1, 0, 0, 1, 193, 88, 0, 4, 0, 21, 4, 0, 0, 0, 128, 4, 0, 21, 0, 0, 0, 7, 128, 76, 0, 0, 0, 0, 0, 47, 0, 0, 0, 199, 0, 2, 0, 180, 180, 79, 117, 1, 0, 0, 0, 51, 0, 0, 0, 97, 114, 109, 116, 97, 114, 103, 101, 116, 51, 47, 54, 46, 57, 46, 49, 56, 49, 49, 50, 55, 79, 83, 83, 47, 52, 50, 99, 55, 99, 97, 102, 47, 52, 50, 99, 55, 99, 97, 102, 47, 47, 97, 114, 109, 118, 55, 108, 46, 49, 0, 0, 1, 0, 0, 0];
        let spdp_parameter_list = parse_spdp_parameter_list(&data).unwrap();

        assert_eq!(spdp_parameter_list.len(), 9);
        assert_eq!(spdp_parameter_list[0], SpdpParameter::ProtocolVersion(ProtocolVersion{major:2, minor:1}));
        assert_eq!(spdp_parameter_list[1], SpdpParameter::VendorId([1,2]));
        assert_eq!(spdp_parameter_list[2], SpdpParameter::DefaultUnicastLocator(Locator::new(1,7411, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 168, 1, 160])));
        assert_eq!(spdp_parameter_list[3], SpdpParameter::DefaultMulticastLocator(Locator::new(1,7401, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1])));
        assert_eq!(spdp_parameter_list[4], SpdpParameter::MetatrafficUnicastLocator(Locator::new(1,7410, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 168, 1, 160])));
        assert_eq!(spdp_parameter_list[5], SpdpParameter::MetatrafficMulticastLocator(Locator::new(1,7400, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1])));
        assert_eq!(spdp_parameter_list[6], SpdpParameter::ParticipantLeaseDuration(Time{seconds:11,fraction:0}));
        assert_eq!(spdp_parameter_list[7], SpdpParameter::ParticipantGuid(GUID::new([6, 85, 244, 87, 0, 0, 0, 4, 0, 0, 0, 1], EntityId::new([0, 0, 1], ENTITY_KIND_BUILT_IN_PARTICIPANT))));
        assert_eq!(spdp_parameter_list[8], SpdpParameter::BuiltinEndpointSet(BuiltInEndPointSet::new(1045)));
    }
}