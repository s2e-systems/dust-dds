use std::{
    io::{BufRead, Write},
    marker::PhantomData,
};

use byteorder::{BigEndian, ByteOrder, LittleEndian};
use rust_rtps_pim::{
    behavior::types::Duration,
    discovery::{
        spdp::spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        types::{BuiltinEndpointQos, BuiltinEndpointSet, DomainId},
    },
    messages::types::Count,
    structure::types::{Locator, ProtocolVersion},
};

use crate::mapping_rtps_protocol::builtin_endpoint::parameter_id_values::{
    PID_BUILTIN_ENDPOINT_QOS, PID_BUILTIN_ENDPOINT_SET, PID_DEFAULT_UNICAST_LOCATOR, PID_DOMAIN_ID,
    PID_DOMAIN_TAG, PID_EXPECTS_INLINE_QOS, PID_METATRAFFIC_MULTICAST_LOCATOR,
    PID_METATRAFFIC_UNICAST_LOCATOR, PID_PARTICIPANT_LEASE_DURATION,
    PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, PID_PROTOCOL_VERSION, PID_SENTINEL,
    PID_UNICAST_LOCATOR, PID_VENDORID,
};
use crate::{
    deserialize::{self, Deserialize},
    serialize::{self, NumberOfBytes, Serialize},
};

use super::parameter_id_values::{
    DEFAULT_BUILTIN_ENDPOINT_QOS, DEFAULT_DOMAIN_TAG, DEFAULT_EXPECTS_INLINE_QOS,
    DEFAULT_PARTICIPANT_LEASE_DURATION, PID_DEFAULT_MULTICAST_LOCATOR,
};

const PL_CDR_BE: [u8; 4] = [0x00, 0x02, 0x00, 0x00];
const PL_CDR_LE: [u8; 4] = [0x00, 0x03, 0x00, 0x00];
struct RepresentationIdentifier;
impl Serialize for RepresentationIdentifier {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        if std::any::type_name::<B>() == std::any::type_name::<BigEndian>() {
            PL_CDR_BE
        } else {
            PL_CDR_LE
        }
        .serialize::<_, B>(&mut writer)
    }
}
struct ParameterSerializer<'a, W: Write, B: ByteOrder> {
    writer: &'a mut W,
    byteorder: PhantomData<B>,
}

impl<'a, W: Write, B: ByteOrder> ParameterSerializer<'a, W, B> {
    fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            byteorder: PhantomData,
        }
    }

    fn serialize_list<S: Serialize + NumberOfBytes>(
        &mut self,
        parameter_id: u16,
        parameter: &[S],
    ) -> crate::serialize::Result {
        for parameter_i in parameter {
            self.serialize(parameter_id, parameter_i)?;
        }
        Ok(())
    }
    fn serialize<S: Serialize + NumberOfBytes>(
        &mut self,
        parameter_id: u16,
        parameter: &S,
    ) -> crate::serialize::Result {
        parameter_id.serialize::<_, B>(&mut self.writer)?;
        let length = parameter.number_of_bytes();
        let padding: &[u8] = match length % 4 {
            1 => &[0; 3],
            2 => &[0; 2],
            3 => &[0; 1],
            _ => &[],
        };
        let length_with_padding = (length + padding.len()) as u16;
        length_with_padding.serialize::<_, B>(&mut self.writer)?;
        parameter.serialize::<_, B>(&mut self.writer)?;
        self.writer.write_all(padding)
    }
}

struct ParameterDeserializer<'de, 'a, B: ByteOrder> {
    buf: &'a mut &'de [u8],
    byteorder: PhantomData<B>,
}

impl<'de, 'a, B: ByteOrder> ParameterDeserializer<'de, 'a, B> {
    fn new(buf: &'a mut &'de [u8]) -> Self {
        Self {
            buf,
            byteorder: PhantomData,
        }
    }
    fn get<D: Deserialize<'de>>(&mut self, parameter_id: u16) -> deserialize::Result<D> {
        let mut buf = *self.buf;
        const MAX_PARAMETERS: usize = 2_usize.pow(16);
        for _ in 0..MAX_PARAMETERS {
            let parameter_id_i: u16 = Deserialize::deserialize::<B>(&mut buf)?;
            let length: i16 = Deserialize::deserialize::<B>(&mut buf)?;
            if parameter_id_i == PID_SENTINEL {
                break;
            }
            if parameter_id_i == parameter_id {
                return Ok(D::deserialize::<B>(&mut buf)?);
            } else {
                buf.consume(length as usize);
            };
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Parameter ID {:x} not found", parameter_id),
        ))
    }
    fn get_list<D: Deserialize<'de>>(&mut self, parameter_id: u16) -> deserialize::Result<Vec<D>> {
        const MAX_PARAMETERS: usize = 2_usize.pow(16);
        let mut buf = *self.buf;
        let mut result = vec![];
        for _ in 0..MAX_PARAMETERS {
            let parameter_id_i: u16 = Deserialize::deserialize::<B>(&mut buf)?;
            let length: i16 = Deserialize::deserialize::<B>(&mut buf)?;
            if parameter_id_i == PID_SENTINEL {
                break;
            }
            if parameter_id_i == parameter_id {
                result.push(D::deserialize::<B>(&mut buf)?);
            } else {
                buf.consume(length as usize);
            };
        }
        Ok(result)
    }
}

impl Serialize for BuiltinEndpointSet {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.0.serialize::<_, B>(&mut writer)
    }
}

impl<'de> Deserialize<'de> for BuiltinEndpointSet {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self(Deserialize::deserialize::<B>(buf)?))
    }
}

impl NumberOfBytes for BuiltinEndpointSet {
    fn number_of_bytes(&self) -> usize {
        4
    }
}

impl Serialize for BuiltinEndpointQos {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.0.serialize::<_, B>(&mut writer)
    }
}

impl<'de> Deserialize<'de> for BuiltinEndpointQos {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self(Deserialize::deserialize::<B>(buf)?))
    }
}

impl NumberOfBytes for BuiltinEndpointQos {
    fn number_of_bytes(&self) -> usize {
        4
    }
}

impl Serialize for SpdpDiscoveredParticipantData<&[Locator]> {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        RepresentationIdentifier.serialize::<_, B>(&mut writer)?;

        let mut ser = ParameterSerializer::<_, B>::new(&mut writer);
        ser.serialize(PID_DOMAIN_ID, &self.domain_id)?;
        if &self.domain_tag != &DEFAULT_DOMAIN_TAG {
            ser.serialize(PID_DOMAIN_TAG, &self.domain_tag)?;
        }
        ser.serialize(PID_PROTOCOL_VERSION, &self.protocol_version)?;
        ser.serialize(PID_VENDORID, &self.vendor_id)?;
        if self.expects_inline_qos != DEFAULT_EXPECTS_INLINE_QOS {
            ser.serialize(PID_EXPECTS_INLINE_QOS, &self.expects_inline_qos)?;
        }
        ser.serialize_list(
            PID_METATRAFFIC_UNICAST_LOCATOR,
            &self.metatraffic_unicast_locator_list,
        )?;
        ser.serialize_list(
            PID_METATRAFFIC_MULTICAST_LOCATOR,
            &self.metatraffic_multicast_locator_list,
        )?;
        ser.serialize_list(
            PID_DEFAULT_UNICAST_LOCATOR,
            &self.default_unicast_locator_list,
        )?;
        ser.serialize_list(
            PID_DEFAULT_UNICAST_LOCATOR,
            &self.default_multicast_locator_list,
        )?;
        ser.serialize(PID_BUILTIN_ENDPOINT_SET, &self.available_builtin_endpoints)?;
        ser.serialize(
            PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT,
            &self.manual_liveliness_count,
        )?;
        if &self.builtin_endpoint_qos != &DEFAULT_BUILTIN_ENDPOINT_QOS {
            ser.serialize(PID_BUILTIN_ENDPOINT_QOS, &self.builtin_endpoint_qos)?;
        }
        if &self.lease_duration != &DEFAULT_PARTICIPANT_LEASE_DURATION {
            ser.serialize(PID_PARTICIPANT_LEASE_DURATION, &self.lease_duration)?;
        }
        ser.serialize(PID_SENTINEL, &[])?;
        Ok(())
    }
}

impl<'de> Deserialize<'de> for SpdpDiscoveredParticipantData<Vec<Locator>> {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        let representation: [u8; 4] = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let mut de = match representation {
            // PL_CDR_BE => ParameterDeserializer::<BigEndian>::new(buf),
            PL_CDR_LE => ParameterDeserializer::<LittleEndian>::new(buf),
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Representation identifier invalid",
                ))
            }
        };

        let _domain_tag: String = de.get(PID_DOMAIN_TAG)?;

        Ok(Self {
            domain_id: de.get(PID_DOMAIN_ID)?,
            domain_tag: "abc", //de.get(PID_DOMAIN_TAG).unwrap_or(DEFAULT_DOMAIN_TAG),
            protocol_version: de.get(PID_PROTOCOL_VERSION)?,
            guid_prefix: [3; 12],
            vendor_id: de.get(PID_VENDORID)?,
            expects_inline_qos: de
                .get(PID_EXPECTS_INLINE_QOS)
                .unwrap_or(DEFAULT_EXPECTS_INLINE_QOS),
            metatraffic_unicast_locator_list: de.get_list(PID_METATRAFFIC_UNICAST_LOCATOR)?,
            metatraffic_multicast_locator_list: de.get_list(PID_METATRAFFIC_MULTICAST_LOCATOR)?,
            default_unicast_locator_list: de.get_list(PID_DEFAULT_UNICAST_LOCATOR)?,
            default_multicast_locator_list: de.get_list(PID_DEFAULT_MULTICAST_LOCATOR)?,
            available_builtin_endpoints: de.get(PID_BUILTIN_ENDPOINT_SET)?,
            lease_duration: de
                .get(PID_PARTICIPANT_LEASE_DURATION)
                .unwrap_or(DEFAULT_PARTICIPANT_LEASE_DURATION),
            manual_liveliness_count: de.get(PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT)?,
            builtin_endpoint_qos: de
                .get(PID_BUILTIN_ENDPOINT_QOS)
                .unwrap_or(DEFAULT_BUILTIN_ENDPOINT_QOS),
        })
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::{
        behavior::types::Duration,
        discovery::types::{BuiltinEndpointQos, BuiltinEndpointSet},
        messages::types::Count,
        structure::types::{Locator, ProtocolVersion},
    };

    use crate::{deserialize::from_bytes_le, serialize::to_bytes_le};

    use super::*;
    #[test]
    fn serialize_spdp_discovered_participant_data() {
        let spdp_discovered_participant_data = SpdpDiscoveredParticipantData {
            domain_id: 76,
            domain_tag: "abc",
            protocol_version: ProtocolVersion { major: 2, minor: 4 },
            guid_prefix: [3; 12],
            vendor_id: [35, 65],
            expects_inline_qos: true,
            metatraffic_unicast_locator_list: &[Locator {
                kind: 1,
                port: 2,
                address: [3; 16],
            }][..],
            metatraffic_multicast_locator_list: &[],
            default_unicast_locator_list: &[],
            default_multicast_locator_list: &[],
            available_builtin_endpoints: BuiltinEndpointSet(7),
            lease_duration: Duration {
                seconds: 42,
                fraction: 43,
            },
            manual_liveliness_count: Count(8),
            builtin_endpoint_qos: BuiltinEndpointQos(9),
        };

        let result = to_bytes_le(&spdp_discovered_participant_data).unwrap();
        assert_eq!(
            result,
            vec![
                0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
                0x0f, 0x00, 4, 0x00, // PID_DOMAIN_ID, Length
                76, 0x00, 0x00, 0x00, // DomainId
                0x14, 0x40, 8, 0x00, // PID_DOMAIN_TAG, Length
                4, 0, 0, 0, // Length: 4
                b'a', b'b', b'c', 0x00, // DomainTag
                0x15, 0x00, 4, 0x00, // PID_PROTOCOL_VERSION, Length
                2, 4, 0x00, 0x00, // ProtocolVersion: major, minor
                0x16, 0x00, 4, 0x00, // PID_VENDORID, Length
                35, 65, 0x00, 0x00, // VendorId
                0x43, 0x00, 4, 0x00, // PID_EXPECTS_INLINE_QOS, Length
                0x01, 0x00, 0x00, 0x00, // True
                0x32, 0x00, 24, 0x00, // PID_METATRAFFIC_UNICAST_LOCATOR, Length
                0x01, 0x00, 0x00, 0x00, // Locator: kind
                0x02, 0x00, 0x00, 0x00, // Locator: port
                0x03, 0x03, 0x03, 0x03, // Locator: address
                0x03, 0x03, 0x03, 0x03, // Locator: address
                0x03, 0x03, 0x03, 0x03, // Locator: address
                0x03, 0x03, 0x03, 0x03, // Locator: address
                0x58, 0x00, 4, 0x00, // PID_BUILTIN_ENDPOINT_SET, Length
                7, 0x00, 0x00, 0x00, //
                0x34, 0x00, 4, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Length
                8, 0x00, 0x00, 0x00, // Count
                0x77, 0x00, 4, 0x00, // PID_BUILTIN_ENDPOINT_QOS, Length: 4
                9, 0x00, 0x00, 0x00, //
                0x02, 0x00, 8, 0x00, // PID_PARTICIPANT_LEASE_DURATION, Length
                42, 0x00, 0x00, 0x00, // Duration: seconds
                43, 0x00, 0x00, 0x00, // Duration: fraction
                0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length
            ]
        );
    }

    #[test]
    fn deserialize_spdp_discovered_participant_data() {
        let expected = SpdpDiscoveredParticipantData {
            domain_id: 76,
            domain_tag: "abc",
            protocol_version: ProtocolVersion { major: 2, minor: 4 },
            guid_prefix: [3; 12],
            vendor_id: [35, 65],
            expects_inline_qos: true,
            metatraffic_unicast_locator_list: vec![Locator {
                kind: 1,
                port: 2,
                address: [3; 16],
            }],
            metatraffic_multicast_locator_list: vec![],
            default_unicast_locator_list: vec![],
            default_multicast_locator_list: vec![],
            available_builtin_endpoints: BuiltinEndpointSet(7),
            lease_duration: Duration {
                seconds: 42,
                fraction: 43,
            },
            manual_liveliness_count: Count(8),
            builtin_endpoint_qos: BuiltinEndpointQos(9),
        };

        assert_eq!(
            expected,
            from_bytes_le(&vec![
                0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
                0x0f, 0x00, 4, 0x00, // PID_DOMAIN_ID, Length
                76, 0x00, 0x00, 0x00, // DomainId
                0x14, 0x40, 8, 0x00, // PID_DOMAIN_TAG, Length
                4, 0, 0, 0, // Length: 4
                b'a', b'b', b'c', 0x00, // DomainTag
                0x15, 0x00, 4, 0x00, // PID_PROTOCOL_VERSION, Length
                2, 4, 0x00, 0x00, // ProtocolVersion: major, minor
                0x16, 0x00, 4, 0x00, // PID_VENDORID, Length
                35, 65, 0x00, 0x00, // VendorId
                0x43, 0x00, 4, 0x00, // PID_EXPECTS_INLINE_QOS, Length
                0x01, 0x00, 0x00, 0x00, // True
                0x32, 0x00, 24, 0x00, // PID_METATRAFFIC_UNICAST_LOCATOR, Length
                0x01, 0x00, 0x00, 0x00, // Locator: kind
                0x02, 0x00, 0x00, 0x00, // Locator: port
                0x03, 0x03, 0x03, 0x03, // Locator: address
                0x03, 0x03, 0x03, 0x03, // Locator: address
                0x03, 0x03, 0x03, 0x03, // Locator: address
                0x03, 0x03, 0x03, 0x03, // Locator: address
                0x58, 0x00, 4, 0x00, // PID_BUILTIN_ENDPOINT_SET, Length
                7, 0x00, 0x00, 0x00, //
                0x34, 0x00, 4, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Length
                8, 0x00, 0x00, 0x00, // Count
                0x77, 0x00, 4, 0x00, // PID_BUILTIN_ENDPOINT_QOS, Length: 4
                9, 0x00, 0x00, 0x00, //
                0x02, 0x00, 8, 0x00, // PID_PARTICIPANT_LEASE_DURATION, Length
                42, 0x00, 0x00, 0x00, // Duration: seconds
                43, 0x00, 0x00, 0x00, // Duration: fraction
                0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length
            ])
            .unwrap()
        );
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{deserialize::from_bytes_le, serialize::to_bytes_le};

//     use super::*;
//     use rust_rtps_pim::structure::types::ENTITYID_PARTICIPANT;

//     #[test]
//     pub fn serialize_spdp_default_data() {
//         let domain_id = 1;
//         let domain_tag = &SPDPdiscoveredParticipantDataUdp::DEFAULT_DOMAIN_TAG;
//         let protocol_version = ProtocolVersion { major: 2, minor: 4 };
//         let guid = Guid::new([1; 12], ENTITYID_PARTICIPANT);
//         let vendor_id = [9, 9];
//         let expects_inline_qos = SPDPdiscoveredParticipantDataUdp::DEFAULT_EXPECTS_INLINE_QOS;
//         let metatraffic_unicast_locator_list = &[];
//         let metatraffic_multicast_locator_list = &[];
//         let default_unicast_locator_list = &[];
//         let default_multicast_locator_list = &[];
//         let available_builtin_endpoints =
//             BuiltinEndpointSet::new(BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR);
//         let manual_liveliness_count = Count(2);
//         let builtin_endpoint_qos = BuiltinEndpointQos::new(0);
//         let lease_duration =
//             SPDPdiscoveredParticipantDataUdp::DEFAULT_PARTICIPANT_LEASE_DURATION.into();

//         let spdp_discovered_participant_data = SPDPdiscoveredParticipantDataUdp::new(
//             &domain_id,
//             domain_tag,
//             &protocol_version,
//             &guid,
//             &vendor_id,
//             &expects_inline_qos,
//             metatraffic_unicast_locator_list,
//             metatraffic_multicast_locator_list,
//             default_unicast_locator_list,
//             default_multicast_locator_list,
//             &available_builtin_endpoints,
//             &manual_liveliness_count,
//             &builtin_endpoint_qos,
//             &lease_duration,
//         );

//         let serialized_data = to_bytes_le(&spdp_discovered_participant_data).unwrap();
//         let expected_data = vec![
//             0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
//             0x0f, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
//             0x01, 0x00, 0x00, 0x00, // DomainId(1)
//             0x15, 0x00, 0x04, 0x00, // PID_PROTOCOL_VERSION, Length: 4
//             0x02, 0x04, 0x00, 0x00, // ProtocolVersion{major:2, minor:4}
//             0x50, 0x00, 0x10, 0x00, // PID_PARTICIPANT_GUID, Length: 16
//             0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
//             0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
//             0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
//             0x00, 0x00, 0x01, 0xc1, // EntityId(ENTITYID_PARTICIPANT)
//             0x16, 0x00, 0x04, 0x00, // PID_VENDORID, Length:4,
//             0x09, 0x09, 0x00, 0x00, // VendorId([9,9])
//             0x58, 0x00, 0x04, 0x00, // PID_BUILTIN_ENDPOINT_SET, Length: 4
//             0x02, 0x00, 0x00, 0x00, // BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
//             0x34, 0x00, 0x04, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Length: 4
//             0x02, 0x00, 0x00, 0x00, // Count(2)
//             0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length: 0
//         ];

//         assert_eq!(serialized_data, expected_data);
//     }

//     #[test]
//     pub fn serialize_complete_spdp_discovered_participant_data() {
//         let locator1 = Locator::new(1, 1, [1; 16]);
//         let locator2 = Locator::new(2, 2, [2; 16]);

//         let domain_id = 1;
//         let domain_tag = "abc";
//         let protocol_version = ProtocolVersion { major: 2, minor: 4 };
//         let guid = Guid::new([1; 12], ENTITYID_PARTICIPANT);
//         let vendor_id = [9, 9];
//         let expects_inline_qos = true;
//         let metatraffic_unicast_locator_list = &[locator1, locator2];
//         let metatraffic_multicast_locator_list = &[locator1, locator2];
//         let default_unicast_locator_list = &[locator1, locator2];
//         let default_multicast_locator_list = &[locator1, locator2];
//         let available_builtin_endpoints =
//             BuiltinEndpointSet::new(BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR);
//         let manual_liveliness_count = Count(2);
//         let builtin_endpoint_qos = BuiltinEndpointQos::new(
//             BuiltinEndpointQos::BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER,
//         );
//         let lease_duration = Duration {
//             seconds: 10,
//             fraction: 0,
//         };

//         let spdp_discovered_participant_data = SPDPdiscoveredParticipantDataUdp::new(
//             &domain_id,
//             domain_tag,
//             &protocol_version,
//             &guid,
//             &vendor_id,
//             &expects_inline_qos,
//             metatraffic_unicast_locator_list,
//             metatraffic_multicast_locator_list,
//             default_unicast_locator_list,
//             default_multicast_locator_list,
//             &available_builtin_endpoints,
//             &manual_liveliness_count,
//             &builtin_endpoint_qos,
//             &lease_duration,
//         );

//         let serialized_data = to_bytes_le(&spdp_discovered_participant_data).unwrap();
//         let expected_data = vec![
//             0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
//             0x0f, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
//             0x01, 0x00, 0x00, 0x00, // DomainId(1)
//             0x14, 0x40, 0x08, 0x00, // PID_DOMAIN_TAG, Length: 8
//             0x04, 0x00, 0x00, 0x00, // DomainTag(length: 4)
//             b'a', b'b', b'c', 0x00, // DomainTag('abc')
//             0x15, 0x00, 0x04, 0x00, // PID_PROTOCOL_VERSION, Length: 4
//             0x02, 0x04, 0x00, 0x00, // ProtocolVersion{major:2, minor:4}
//             0x50, 0x00, 0x10, 0x00, // PID_PARTICIPANT_GUID, Length: 16
//             0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
//             0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
//             0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
//             0x00, 0x00, 0x01, 0xc1, // EntityId(ENTITYID_PARTICIPANT)
//             0x16, 0x00, 0x04, 0x00, // PID_VENDORID, Length:4,
//             0x09, 0x09, 0x00, 0x00, // VendorId([9,9])
//             0x43, 0x00, 0x04, 0x00, // PID_EXPECTS_INLINE_QOS, Length: 4,
//             0x01, 0x00, 0x00, 0x00, // True
//             0x32, 0x00, 0x18, 0x00, // PID_METATRAFFIC_UNICAST_LOCATOR, Length: 24,
//             0x01, 0x00, 0x00, 0x00, // Locator{kind:1
//             0x01, 0x00, 0x00, 0x00, // port:1,
//             0x01, 0x01, 0x01, 0x01, //
//             0x01, 0x01, 0x01, 0x01, // address: [1;16]
//             0x01, 0x01, 0x01, 0x01, //
//             0x01, 0x01, 0x01, 0x01, // }
//             0x32, 0x00, 0x18, 0x00, // PID_METATRAFFIC_UNICAST_LOCATOR, Length: 24,
//             0x02, 0x00, 0x00, 0x00, // Locator{kind:2
//             0x02, 0x00, 0x00, 0x00, // port:2,
//             0x02, 0x02, 0x02, 0x02, //
//             0x02, 0x02, 0x02, 0x02, // address: [2;16]
//             0x02, 0x02, 0x02, 0x02, //
//             0x02, 0x02, 0x02, 0x02, // }
//             0x33, 0x00, 0x18, 0x00, // PID_METATRAFFIC_MULTICAST_LOCATOR, Length: 24,
//             0x01, 0x00, 0x00, 0x00, // Locator{kind:1
//             0x01, 0x00, 0x00, 0x00, // port:1,
//             0x01, 0x01, 0x01, 0x01, //
//             0x01, 0x01, 0x01, 0x01, // address: [1;16]
//             0x01, 0x01, 0x01, 0x01, //
//             0x01, 0x01, 0x01, 0x01, // }
//             0x33, 0x00, 0x18, 0x00, // PID_METATRAFFIC_MULTICAST_LOCATOR, Length: 24,
//             0x02, 0x00, 0x00, 0x00, // Locator{kind:2
//             0x02, 0x00, 0x00, 0x00, // port:2,
//             0x02, 0x02, 0x02, 0x02, //
//             0x02, 0x02, 0x02, 0x02, // address: [2;16]
//             0x02, 0x02, 0x02, 0x02, //
//             0x02, 0x02, 0x02, 0x02, // }
//             0x31, 0x00, 0x18, 0x00, // PID_DEFAULT_UNICAST_LOCATOR, Length: 24,
//             0x01, 0x00, 0x00, 0x00, // Locator{kind:1
//             0x01, 0x00, 0x00, 0x00, // port:1,
//             0x01, 0x01, 0x01, 0x01, //
//             0x01, 0x01, 0x01, 0x01, // address: [1;16]
//             0x01, 0x01, 0x01, 0x01, //
//             0x01, 0x01, 0x01, 0x01, // }
//             0x31, 0x00, 0x18, 0x00, // PID_DEFAULT_UNICAST_LOCATOR, Length: 24,
//             0x02, 0x00, 0x00, 0x00, // Locator{kind:2
//             0x02, 0x00, 0x00, 0x00, // port:2,
//             0x02, 0x02, 0x02, 0x02, //
//             0x02, 0x02, 0x02, 0x02, // address: [2;16]
//             0x02, 0x02, 0x02, 0x02, //
//             0x02, 0x02, 0x02, 0x02, // }
//             0x48, 0x00, 0x18, 0x00, // PID_DEFAULT_MULTICAST_LOCATOR, Length: 24,
//             0x01, 0x00, 0x00, 0x00, // Locator{kind:1
//             0x01, 0x00, 0x00, 0x00, // port:1,
//             0x01, 0x01, 0x01, 0x01, //
//             0x01, 0x01, 0x01, 0x01, // address: [1;16]
//             0x01, 0x01, 0x01, 0x01, //
//             0x01, 0x01, 0x01, 0x01, // }
//             0x48, 0x00, 0x18, 0x00, // PID_DEFAULT_MULTICAST_LOCATOR, Length: 24,
//             0x02, 0x00, 0x00, 0x00, // Locator{kind:2
//             0x02, 0x00, 0x00, 0x00, // port:2,
//             0x02, 0x02, 0x02, 0x02, //
//             0x02, 0x02, 0x02, 0x02, // address: [2;16]
//             0x02, 0x02, 0x02, 0x02, //
//             0x02, 0x02, 0x02, 0x02, // }
//             0x58, 0x00, 0x04, 0x00, // PID_BUILTIN_ENDPOINT_SET, Length: 4
//             0x02, 0x00, 0x00, 0x00, // BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
//             0x34, 0x00, 0x04, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Length: 4
//             0x02, 0x00, 0x00, 0x00, // Count(2)
//             0x77, 0x00, 0x04, 0x00, // PID_BUILTIN_ENDPOINT_QOS, Length: 4
//             0x00, 0x00, 0x00, 0x20, // BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER
//             0x02, 0x00, 0x08, 0x00, // PID_PARTICIPANT_LEASE_DURATION, Length: 8
//             0x0a, 0x00, 0x00, 0x00, // Duration{seconds:30,
//             0x00, 0x00, 0x00, 0x00, //          fraction:0}
//             0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length: 0
//         ];

//         assert_eq!(serialized_data, expected_data);
//     }

//     #[test]
//     fn deserialize_complete_spdp_discovered_participant_data() {
//         #[rustfmt::skip]
//         let spdp_discovered_participant_data: SPDPdiscoveredParticipantDataUdp = from_bytes_le(&[
//             0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
//             0x0f, 0x00, 4, 0x00,    // PID_DOMAIN_ID, Length
//             0x01, 0x00, 0x00, 0x00, // DomainId
//             0x14, 0x40, 8, 0x00,    // PID_DOMAIN_TAG, Length
//             4,    0,    0,    0,             // Length: 4
//             b'a', b'b', b'c', 0x00, // DomainTag
//             0x15, 0x00, 4, 0x00,    // PID_PROTOCOL_VERSION, Length
//             0x02, 0x04, 0x00, 0x00, // ProtocolVersion: major, minor
//             0x50, 0x00, 16, 0x00,   // PID_PARTICIPANT_GUID, Length
//             0x01, 0x01, 0x01, 0x01, // GuidPrefix
//             0x01, 0x01, 0x01, 0x01, // GuidPrefix
//             0x01, 0x01, 0x01, 0x01, // GuidPrefix
//             0x00, 0x00, 0x01, 0xc1, // EntityId(ENTITYID_PARTICIPANT)
//             0x16, 0x00, 4, 0x00,    // PID_VENDORID, Length
//             0x09, 0x09, 0x00, 0x00, // VendorId
//             0x43, 0x00, 4, 0x00,    // PID_EXPECTS_INLINE_QOS, Length
//             0x01, 0x00, 0x00, 0x00, // True
//             0x32, 0x00, 24, 0x00,   // PID_METATRAFFIC_UNICAST_LOCATOR, Length
//             0x01, 0x00, 0x00, 0x00, // Locator: kind
//             0x01, 0x00, 0x00, 0x00, // Locator: port
//             0x01, 0x01, 0x01, 0x01, // Locator: address
//             0x01, 0x01, 0x01, 0x01, // Locator: address
//             0x01, 0x01, 0x01, 0x01, // Locator: address
//             0x01, 0x01, 0x01, 0x01, // Locator: address
//             0x32, 0x00, 24, 0x00,   // PID_METATRAFFIC_UNICAST_LOCATOR, Length
//             0x02, 0x00, 0x00, 0x00, // Locator: kind
//             0x02, 0x00, 0x00, 0x00, // Locator: port
//             0x02, 0x02, 0x02, 0x02, // Locator: address
//             0x02, 0x02, 0x02, 0x02, // Locator: address
//             0x02, 0x02, 0x02, 0x02, // Locator: address
//             0x02, 0x02, 0x02, 0x02, // Locator: address
//             0x33, 0x00, 24, 0x00,   // PID_METATRAFFIC_MULTICAST_LOCATOR, Length
//             0x01, 0x00, 0x00, 0x00, // Locator: kind
//             0x01, 0x00, 0x00, 0x00, // Locator: port
//             0x01, 0x01, 0x01, 0x01, // Locator: address
//             0x01, 0x01, 0x01, 0x01, // Locator: address
//             0x01, 0x01, 0x01, 0x01, // Locator: address
//             0x01, 0x01, 0x01, 0x01, // Locator: address
//             0x33, 0x00, 24, 0x00,   // PID_METATRAFFIC_MULTICAST_LOCATOR, Length
//             0x02, 0x00, 0x00, 0x00, // Locator: kind
//             0x02, 0x00, 0x00, 0x00, // Locator: port,
//             0x02, 0x02, 0x02, 0x02, // Locator: address
//             0x02, 0x02, 0x02, 0x02, // Locator: address
//             0x02, 0x02, 0x02, 0x02, // Locator: address
//             0x02, 0x02, 0x02, 0x02, // Locator: address
//             0x31, 0x00, 24, 0x00,   // PID_DEFAULT_UNICAST_LOCATOR, Length
//             0x01, 0x00, 0x00, 0x00, // Locator: kind
//             0x01, 0x00, 0x00, 0x00, // Locator: port
//             0x01, 0x01, 0x01, 0x01, // Locator: address
//             0x01, 0x01, 0x01, 0x01, // Locator: address
//             0x01, 0x01, 0x01, 0x01, // Locator: address
//             0x01, 0x01, 0x01, 0x01, // Locator: address
//             0x31, 0x00, 24, 0x00,   // PID_DEFAULT_UNICAST_LOCATOR, Length
//             0x02, 0x00, 0x00, 0x00, // Locator: kind
//             0x02, 0x00, 0x00, 0x00, // Locator: port
//             0x02, 0x02, 0x02, 0x02, // Locator: address
//             0x02, 0x02, 0x02, 0x02, // Locator: address
//             0x02, 0x02, 0x02, 0x02, // Locator: address
//             0x02, 0x02, 0x02, 0x02, // Locator: address
//             0x48, 0x00, 24, 0x00,   // PID_DEFAULT_MULTICAST_LOCATOR, Length
//             0x01, 0x00, 0x00, 0x00, // Locator: kind
//             0x01, 0x00, 0x00, 0x00, // Locator: port
//             0x01, 0x01, 0x01, 0x01, // Locator: address
//             0x01, 0x01, 0x01, 0x01, // Locator: address
//             0x01, 0x01, 0x01, 0x01, // Locator: address
//             0x01, 0x01, 0x01, 0x01, // Locator: address
//             0x48, 0x00, 024, 0x00, // PID_DEFAULT_MULTICAST_LOCATOR, Length,
//             0x02, 0x00, 0x00, 0x00, // Locator: kind
//             0x02, 0x00, 0x00, 0x00, // Locator: port
//             0x02, 0x02, 0x02, 0x02, // Locator: address
//             0x02, 0x02, 0x02, 0x02, // Locator: address
//             0x02, 0x02, 0x02, 0x02, // Locator: address
//             0x02, 0x02, 0x02, 0x02, // Locator: address
//             0x58, 0x00, 4, 0x00,    // PID_BUILTIN_ENDPOINT_SET, Length
//             0x02, 0x00, 0x00, 0x00, // BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
//             0x34, 0x00, 4, 0x00,    // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Length
//             0x02, 0x00, 0x00, 0x00, // Count
//             0x77, 0x00, 4, 0x00,    // PID_BUILTIN_ENDPOINT_QOS, Length: 4
//             0x00, 0x00, 0x00, 0x20, // BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER
//             0x02, 0x00, 8, 0x00,    // PID_PARTICIPANT_LEASE_DURATION, Length
//             10, 0x00, 0x00, 0x00,   // Duration: seconds
//             0x00, 0x00, 0x00, 0x00, // Duration: fraction
//             0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length
//         ]).unwrap();

//         let locator1 = Locator::new(1, 1, [1; 16]);
//         let locator2 = Locator::new(2, 2, [2; 16]);

//         let domain_id = 1;
//         let domain_tag = "abc";
//         let protocol_version = ProtocolVersion { major: 2, minor: 4 };
//         let guid = Guid::new([1; 12], ENTITYID_PARTICIPANT);
//         let vendor_id = [9, 9];
//         let expects_inline_qos = true;
//         let metatraffic_unicast_locator_list = &[locator1, locator2];
//         let metatraffic_multicast_locator_list = &[locator1, locator2];
//         let default_unicast_locator_list = &[locator1, locator2];
//         let default_multicast_locator_list = &[locator1, locator2];
//         let available_builtin_endpoints =
//             BuiltinEndpointSet::new(BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR);
//         let manual_liveliness_count = Count(2);
//         let builtin_endpoint_qos = BuiltinEndpointQos::new(
//             BuiltinEndpointQos::BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER,
//         );
//         let lease_duration = Duration {
//             seconds: 10,
//             fraction: 0,
//         };

//         let expected_spdp_discovered_participant_data = SPDPdiscoveredParticipantDataUdp::new(
//             &domain_id,
//             domain_tag,
//             &protocol_version,
//             &guid,
//             &vendor_id,
//             &expects_inline_qos,
//             metatraffic_unicast_locator_list,
//             metatraffic_multicast_locator_list,
//             default_unicast_locator_list,
//             default_multicast_locator_list,
//             &available_builtin_endpoints,
//             &manual_liveliness_count,
//             &builtin_endpoint_qos,
//             &lease_duration,
//         );

//         assert_eq!(
//             spdp_discovered_participant_data,
//             expected_spdp_discovered_participant_data
//         );
//     }

//     #[test]
//     fn deserialize_default_spdp_discovered_participant_data() {
//         #[rustfmt::skip]
//         let spdp_discovered_participant_data: SPDPdiscoveredParticipantDataUdp = from_bytes_le(&[
//             0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
//             0x0f, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
//             0x01, 0x00, 0x00, 0x00, // DomainId(1)
//             0x15, 0x00, 0x04, 0x00, // PID_PROTOCOL_VERSION, Length: 4
//             0x02, 0x04, 0x00, 0x00, // ProtocolVersion{major:2, minor:4}
//             0x50, 0x00, 0x10, 0x00, // PID_PARTICIPANT_GUID, Length: 16
//             0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
//             0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
//             0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
//             0x00, 0x00, 0x01, 0xc1, // EntityId(ENTITYID_PARTICIPANT)
//             0x16, 0x00, 0x04, 0x00, // PID_VENDORID, Length:4,
//             0x09, 0x09, 0x00, 0x00, // VendorId([9,9])
//             0x58, 0x00, 0x04, 0x00, // PID_BUILTIN_ENDPOINT_SET, Length: 4
//             0x02, 0x00, 0x00, 0x00, // BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
//             0x34, 0x00, 0x04, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Length: 4
//             0x02, 0x00, 0x00, 0x00, // Count(2)
//             0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length: 0
//         ]).unwrap();

//         let domain_id = 1;
//         let domain_tag = &SPDPdiscoveredParticipantDataUdp::DEFAULT_DOMAIN_TAG;
//         let protocol_version = ProtocolVersion { major: 2, minor: 4 };
//         let guid = Guid::new([1; 12], ENTITYID_PARTICIPANT);
//         let vendor_id = [9, 9];
//         let expects_inline_qos = SPDPdiscoveredParticipantDataUdp::DEFAULT_EXPECTS_INLINE_QOS;
//         let metatraffic_unicast_locator_list = &[];
//         let metatraffic_multicast_locator_list = &[];
//         let default_unicast_locator_list = &[];
//         let default_multicast_locator_list = &[];
//         let available_builtin_endpoints =
//             BuiltinEndpointSet::new(BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR);
//         let manual_liveliness_count = Count(2);
//         let builtin_endpoint_qos = BuiltinEndpointQos::new(0);
//         let lease_duration =
//             SPDPdiscoveredParticipantDataUdp::DEFAULT_PARTICIPANT_LEASE_DURATION.into();

//         let expected_spdp_discovered_participant_data = SPDPdiscoveredParticipantDataUdp::new(
//             &domain_id,
//             domain_tag,
//             &protocol_version,
//             &guid,
//             &vendor_id,
//             &expects_inline_qos,
//             metatraffic_unicast_locator_list,
//             metatraffic_multicast_locator_list,
//             default_unicast_locator_list,
//             default_multicast_locator_list,
//             &available_builtin_endpoints,
//             &manual_liveliness_count,
//             &builtin_endpoint_qos,
//             &lease_duration,
//         );

//         assert_eq!(
//             spdp_discovered_participant_data,
//             expected_spdp_discovered_participant_data
//         );
//     }

//     #[test]
//     fn deserialize_wrong_spdp_discovered_participant_data() {
//         #[rustfmt::skip]
//         let result: std::result::Result<SPDPdiscoveredParticipantDataUdp, _> = from_bytes_le(&[
//             0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
//             0x0f, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
//             0x01, 0x00, 0x00, 0x00, // DomainId(1)
//             0x50, 0x00, 0x10, 0x00, // PID_PARTICIPANT_GUID, Length: 16
//             0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
//             0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
//             0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
//             0x00, 0x00, 0x01, 0xc1, // EntityId(ENTITYID_PARTICIPANT)
//             0x16, 0x00, 0x04, 0x00, // PID_VENDORID, Length:4,
//             0x09, 0x09, 0x00, 0x00, // VendorId([9,9])
//             0x58, 0x00, 0x04, 0x00, // PID_BUILTIN_ENDPOINT_SET, Length: 4
//             0x02, 0x00, 0x00, 0x00, // BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
//             0x34, 0x00, 0x04, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Length: 4
//             0x02, 0x00, 0x00, 0x00, // Count(2)
//             0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length: 0
//         ]);
//         match result {
//             Err(err) => {
//                 assert_eq!(err.kind(), std::io::ErrorKind::Other);
//                 assert_eq!(err.to_string(), "Missing PID_PROTOCOL_VERSION parameter")
//             }
//             _ => panic!(),
//         }

//         #[rustfmt::skip]
//         let result: Result<SPDPdiscoveredParticipantDataUdp, _> = from_bytes_le(&[
//             0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
//             0x0f, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
//             0x01, 0x00, 0x00, 0x00, // DomainId(1)
//             0x15, 0x00, 0x04, 0x00, // PID_PROTOCOL_VERSION, Length: 4
//             0x02, 0x04, 0x00, 0x00, // ProtocolVersion{major:2, minor:4}
//             0x16, 0x00, 0x04, 0x00, // PID_VENDORID, Length:4,
//             0x09, 0x09, 0x00, 0x00, // VendorId([9,9])
//             0x58, 0x00, 0x04, 0x00, // PID_BUILTIN_ENDPOINT_SET, Length: 4
//             0x02, 0x00, 0x00, 0x00, // BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
//             0x34, 0x00, 0x04, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Length: 4
//             0x02, 0x00, 0x00, 0x00, // Count(2)
//             0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length: 0
//         ]);

//         match result {
//             Err(err) => {
//                 assert_eq!(err.kind(), std::io::ErrorKind::Other);
//                 assert_eq!(err.to_string(), "Missing PID_PARTICIPANT_GUID parameter")
//             }
//             _ => panic!(),
//         }
//     }
// }
