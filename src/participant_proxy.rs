use crate::types::{GuidPrefix, LocatorList, ProtocolVersion, VendorId};
// use crate::spdp::{SpdpParameter,SpdpParameterList, SpdpParameterId};

use num_derive::FromPrimitive;

use crate::messages;
use crate::messages::helpers::{deserialize, EndianessFlag};
use crate::types::{BuiltInEndPointSet, Locator, Time, GUID};

#[derive(Debug)]
pub enum SpdpErrorMessage {
    DataMessageTooSmall,
    InvalidEndianess,
    ParameterLengthTooSmall,
    InvalidParameterLength,
    ParameterContentConversionFailed,
    CdrError(cdr::Error),
    ParserError(messages::RtpsMessageError),
}

impl From<cdr::Error> for SpdpErrorMessage {
    fn from(error: cdr::Error) -> Self {
        SpdpErrorMessage::CdrError(error)
    }
}

impl From<messages::RtpsMessageError> for SpdpErrorMessage {
    fn from(error: messages::RtpsMessageError) -> Self {
        SpdpErrorMessage::ParserError(error)
    }
}

#[derive(FromPrimitive, Eq, PartialEq, Hash, Debug)]
pub enum SpdpParameterId {
    Pad = 0x0000,                              //N/A
    Sentinel = 0x0001,                         //N/A
    UserData = 0x002c,                         //UserDataQosPolicy
    TopicName = 0x0005,                        //string<256>
    TypeName = 0x0007,                         //string<256>
    GroupData = 0x002d,                        //GroupDataQosPolicy
    TopicData = 0x002e,                        //TopicDataQosPolicy
    Durability = 0x001d,                       //DurabilityQosPolicy
    DurabilityService = 0x001e,                //DurabilityServiceQosPolicy
    Deadline = 0x0023,                         //DeadlineQosPolicy
    LatencyBudget = 0x0027,                    //LatencyBudgetQosPolicy
    Liveliness = 0x001b,                       //LivelinessQosPolicy
    Reliability = 0x001a,                      //ReliabilityQosPolicy
    Lifespan = 0x002b,                         //LifespanQosPolicy
    DestinationOrder = 0x0025,                 //DestinationOrderQosPolicy
    History = 0x0040,                          //HistoryQosPolicy
    ResourceLimits = 0x0041,                   //ResourceLimitsQosPolicy
    Ownership = 0x001f,                        //OwnershipQosPolicy
    OwnershipStrength = 0x0006,                //OwnershipStrengthQosPolicy
    Presentation = 0x0021,                     //PresentationQosPolicy
    Partition = 0x0029,                        //PartitionQosPolicy
    TimeBasedFilter = 0x0004,                  //TimeBasedFilterQosPolicy
    TransportPriority = 0x0049,                //TransportPriorityQoSPolicy
    DomainId = 0x000f,                         //DomainId_t
    DomainTag = 0x4014,                        //string<256>
    ProtocolVersion = 0x0015,                  //ProtocolVersion_t
    VendorId = 0x0016,                         //VendorId_t
    UnicastLocator = 0x002f,                   //Locator_t
    MulticastLocator = 0x0030,                 //Locator_t
    DefaultUnicastLocator = 0x0031,            //Locator_t
    DefaultMulticastLocator = 0x0048,          //Locator_t
    MetatrafficUnicastLocator = 0x0032,        //Locator_t
    MetatrafficMulticastLocator = 0x0033,      //Locator_t
    ExpectsInlineQoS = 0x0043,                 //boolean
    ParticipantManualLivelinessCount = 0x0034, //Count_t
    ParticipantLeaseDuration = 0x0002,         //Duration_t
    ContentFilterProperty = 0x0035,            //ContentFilterProperty_t
    ParticipantGuid = 0x0050,                  //GUID_t
    GroupGuid = 0x0052,                        //GUID_t
    BuiltinEndpointSet = 0x0058,               //BuiltinEndpointSet_t
    BuiltinEndpointQoS = 0x0077,               //BuiltinEndpointQos_t
    PropertyList = 0x0059,                     //sequence<Property_t>
    TypeMaxSizeSerialized = 0x0060,            //long
    EntityName = 0x0062,                       //EntityName_t
    EndpointGuid = 0x005a,                     //GUID_t
}

type Result<T> = std::result::Result<T, SpdpErrorMessage>;

#[derive(PartialEq, Eq, Hash)]
pub struct ParticipantProxy {
    domain_id: u32, // TODO: Create DomainId type
    domain_tag: String,
    protocol_version: ProtocolVersion,
    guid_prefix: GuidPrefix,
    vendor_id: VendorId,
    expects_inline_qos: bool,
    available_builtin_endpoints: BuiltInEndPointSet,
    //TODO: Built-in endpoints qos
    metatraffic_unicast_locator_list: LocatorList,
    metatraffic_multicast_locator_list: LocatorList,
    default_multicast_locator_list: LocatorList,
    default_unicast_locator_list: LocatorList,
    //TODO: manual liveliness count
    lease_duration: Time,
}

impl ParticipantProxy {
    pub fn new(
        domain_id: u32, // TODO: Create DomainId type
        domain_tag: String,
        protocol_version: ProtocolVersion,
        guid_prefix: GuidPrefix,
        vendor_id: VendorId,
        expects_inline_qos: bool,
        available_builtin_endpoints: BuiltInEndPointSet,
        //TODO: Built-in endpoints qos
        metatraffic_unicast_locator_list: LocatorList,
        metatraffic_multicast_locator_list: LocatorList,
        default_multicast_locator_list: LocatorList,
        default_unicast_locator_list: LocatorList,
        //TODO: manual liveliness count
        lease_duration: Time,
    ) -> Self {
        ParticipantProxy {
            domain_id,
            domain_tag,
            protocol_version,
            guid_prefix,
            vendor_id,
            expects_inline_qos,
            available_builtin_endpoints,
            //TODO: Built-in endpoints qos
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            default_multicast_locator_list,
            default_unicast_locator_list,
            //TODO: manual liveliness count
            lease_duration,
        }
    }

    pub fn new_from_data(data: &[u8]) -> Result<Self> {
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
        // Defaults:
        let mut protocol_version = ProtocolVersion { major: 2, minor: 4 };
        let mut guid_prefix: GuidPrefix = [1; 12];
        let mut vendor_id: VendorId = [1, 2];
        let mut expects_inline_qos = false;
        let mut available_builtin_endpoints = BuiltInEndPointSet::new(0);
        let mut metatraffic_unicast_locator_list = LocatorList::new();
        let mut metatraffic_multicast_locator_list = LocatorList::new();
        let mut default_multicast_locator_list = LocatorList::new();
        let mut default_unicast_locator_list = LocatorList::new();
        let mut lease_duration = Time {
            seconds: 30,
            fraction: 0,
        };

        let mut parameter_id_first_index = FIRST_PARAMETER_LENGTH_OFFSET;

        loop {
            let parameter_id_last_index = parameter_id_first_index + PARAMETER_ID_OFFSET;
            let length_first_index = parameter_id_first_index + LENGTH_FIRST_OFFSET;
            let length_last_index = parameter_id_first_index + LENGTH_LAST_OFFSET;

            let value_first_index = parameter_id_first_index + VALUE_FIRST_OFFSET;
            let value_last_index;

            let parameter_id = deserialize::<u16>(
                data,
                &parameter_id_first_index,
                &parameter_id_last_index,
                &endianess,
            )?;

            if parameter_id == SpdpParameterId::Sentinel as u16 {
                break;
            }

            let length =
                deserialize::<u16>(data, &length_first_index, &length_last_index, &endianess)?
                    as usize;
            if length < MINIMUM_PARAMETER_VALUE_LENGTH {
                return Err(SpdpErrorMessage::ParameterLengthTooSmall);
            }

            value_last_index = value_first_index + length - 1;
            if value_last_index >= data.len() {
                return Err(SpdpErrorMessage::InvalidParameterLength);
            }
            let parameter_id_type = num::FromPrimitive::from_u16(parameter_id);
            match parameter_id_type {
                Some(SpdpParameterId::ProtocolVersion) => {
                    protocol_version = deserialize::<ProtocolVersion>(
                        &data,
                        &value_first_index,
                        &value_last_index,
                        &endianess,
                    )?
                }
                Some(SpdpParameterId::VendorId) => {
                    vendor_id = deserialize::<VendorId>(
                        &data,
                        &value_first_index,
                        &value_last_index,
                        &endianess,
                    )?
                }
                Some(SpdpParameterId::DefaultUnicastLocator) => {
                    default_unicast_locator_list.push(deserialize::<Locator>(
                        &data,
                        &value_first_index,
                        &value_last_index,
                        &endianess,
                    )?)
                }
                Some(SpdpParameterId::DefaultMulticastLocator) => default_multicast_locator_list
                    .push(deserialize::<Locator>(
                        &data,
                        &value_first_index,
                        &value_last_index,
                        &endianess,
                    )?),
                Some(SpdpParameterId::MetatrafficUnicastLocator) => {
                    metatraffic_unicast_locator_list.push(deserialize::<Locator>(
                        &data,
                        &value_first_index,
                        &value_last_index,
                        &endianess,
                    )?)
                }
                Some(SpdpParameterId::MetatrafficMulticastLocator) => {
                    metatraffic_multicast_locator_list.push(deserialize::<Locator>(
                        &data,
                        &value_first_index,
                        &value_last_index,
                        &endianess,
                    )?)
                }
                Some(SpdpParameterId::ParticipantLeaseDuration) => {
                    lease_duration = deserialize::<Time>(
                        &data,
                        &value_first_index,
                        &value_last_index,
                        &endianess,
                    )?
                }
                Some(SpdpParameterId::ParticipantGuid) => {
                    guid_prefix = *deserialize::<GUID>(
                        &data,
                        &value_first_index,
                        &value_last_index,
                        &endianess,
                    )?
                    .prefix()
                }
                Some(SpdpParameterId::BuiltinEndpointSet) => {
                    available_builtin_endpoints = deserialize::<BuiltInEndPointSet>(
                        &data,
                        &value_first_index,
                        &value_last_index,
                        &endianess,
                    )?
                }
                Some(SpdpParameterId::ExpectsInlineQoS) => {
                    expects_inline_qos = deserialize::<bool>(
                        &data,
                        &value_first_index,
                        &value_last_index,
                        &endianess,
                    )?
                }
                _ => (),
            };

            parameter_id_first_index = value_last_index + 1;
        }

        Ok(ParticipantProxy {
            domain_id: 1,
            domain_tag: String::from(""),
            protocol_version,
            guid_prefix,
            available_builtin_endpoints,
            vendor_id,
            expects_inline_qos,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            default_multicast_locator_list,
            default_unicast_locator_list,
            lease_duration,
        })
    }

    pub fn available_builtin_endpoints(&self) -> &BuiltInEndPointSet {
        &self.available_builtin_endpoints
    }

    pub fn guid_prefix(&self) -> &GuidPrefix {
        &self.guid_prefix
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    pub fn metatraffic_unicast_locator_list(&self) -> &LocatorList {
        &self.metatraffic_unicast_locator_list
    }

    pub fn metatraffic_multicast_locator_list(&self) -> &LocatorList {
        &self.metatraffic_multicast_locator_list
    }

    pub fn default_unicast_locator_list(&self) -> &LocatorList {
        &self.default_unicast_locator_list
    }

    pub fn default_multicast_locator_list(&self) -> &LocatorList {
        &self.default_multicast_locator_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BuiltInEndPoints};

    #[test]
    fn test_parse_example_spdp_data() {
        let data = [
            0, 3, 0, 0, 21, 0, 4, 0, 2, 1, 0, 0, 22, 0, 4, 0, 1, 2, 0, 0, 49, 0, 24, 0, 1, 0, 0, 0,
            243, 28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 168, 1, 160, 72, 0, 24, 0, 1,
            0, 0, 0, 233, 28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1, 50, 0, 24,
            0, 1, 0, 0, 0, 242, 28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 168, 1, 160, 51,
            0, 24, 0, 1, 0, 0, 0, 232, 28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0,
            1, 2, 0, 8, 0, 11, 0, 0, 0, 0, 0, 0, 0, 80, 0, 16, 0, 6, 85, 244, 87, 0, 0, 0, 4, 0, 0,
            0, 1, 0, 0, 1, 193, 88, 0, 4, 0, 21, 4, 0, 0, 0, 128, 4, 0, 21, 0, 0, 0, 7, 128, 76, 0,
            0, 0, 0, 0, 47, 0, 0, 0, 199, 0, 2, 0, 180, 180, 79, 117, 1, 0, 0, 0, 51, 0, 0, 0, 97,
            114, 109, 116, 97, 114, 103, 101, 116, 51, 47, 54, 46, 57, 46, 49, 56, 49, 49, 50, 55,
            79, 83, 83, 47, 52, 50, 99, 55, 99, 97, 102, 47, 52, 50, 99, 55, 99, 97, 102, 47, 47,
            97, 114, 109, 118, 55, 108, 46, 49, 0, 0, 1, 0, 0, 0,
        ];
        let participant_proxy = ParticipantProxy::new_from_data(&data).unwrap();

        assert_eq!(
            participant_proxy.protocol_version,
            ProtocolVersion { major: 2, minor: 1 }
        );
        assert_eq!(participant_proxy.vendor_id, [1, 2]);
        assert!(participant_proxy
            .available_builtin_endpoints
            .has(BuiltInEndPoints::ParticipantAnnouncer));
        assert!(participant_proxy
            .available_builtin_endpoints
            .has(BuiltInEndPoints::PublicationsAnnouncer));
        assert!(participant_proxy
            .available_builtin_endpoints
            .has(BuiltInEndPoints::SubscriptionsAnnouncer));
        assert!(participant_proxy
            .available_builtin_endpoints
            .has(BuiltInEndPoints::ParticipantMessageDataWriter));
        assert!(!participant_proxy
            .available_builtin_endpoints
            .has(BuiltInEndPoints::ParticipantMessageDataReader));
        assert_eq!(
            participant_proxy.default_unicast_locator_list,
            vec!(Locator::new(
                1,
                7411,
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 168, 1, 160]
            ))
        );
        assert_eq!(
            participant_proxy.default_multicast_locator_list,
            vec!(Locator::new(
                1,
                7401,
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1]
            ))
        );
        assert_eq!(
            participant_proxy.metatraffic_unicast_locator_list,
            vec!(Locator::new(
                1,
                7410,
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 168, 1, 160]
            ))
        );
        assert_eq!(
            participant_proxy.metatraffic_multicast_locator_list,
            vec!(Locator::new(
                1,
                7400,
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1]
            ))
        );
        assert_eq!(
            participant_proxy.lease_duration,
            Time {
                seconds: 11,
                fraction: 0
            }
        );
        assert_eq!(
            participant_proxy.guid_prefix,
            [6, 85, 244, 87, 0, 0, 0, 4, 0, 0, 0, 1]
        );
    }
}
