use std::{io::Write, marker::PhantomData};

use byteorder::{BigEndian, ByteOrder};
use rust_rtps_pim::{
    behavior::types::Duration,
    discovery::{
        spdp::spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        types::{BuiltinEndpointQos, BuiltinEndpointSet},
    },
    structure::types::Locator,
};

use crate::{
    builtin_endpoints::parameterid_list::{
        PID_BUILTIN_ENDPOINT_QOS, PID_BUILTIN_ENDPOINT_SET, PID_DEFAULT_UNICAST_LOCATOR,
        PID_DOMAIN_ID, PID_DOMAIN_TAG, PID_EXPECTS_INLINE_QOS, PID_METATRAFFIC_MULTICAST_LOCATOR,
        PID_METATRAFFIC_UNICAST_LOCATOR, PID_PARTICIPANT_LEASE_DURATION,
        PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, PID_PROTOCOL_VERSION, PID_SENTINEL, PID_VENDORID,
    },
    serialize::{self, NumberOfBytes, Serialize},
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

// Constant value from Table 9.14 - ParameterId mapping and default values
const DEFAULT_DOMAIN_TAG: String = String::new();
const DEFAULT_EXPECTS_INLINE_QOS: bool = false;
const DEFAULT_BUILTIN_ENDPOINT_QOS: BuiltinEndpointQos = BuiltinEndpointQos(0);
const DEFAULT_PARTICIPANT_LEASE_DURATION: Duration = Duration {
    seconds: 100,
    fraction: 0,
};

impl Serialize for BuiltinEndpointSet {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.0.serialize::<_, B>(&mut writer)
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
impl NumberOfBytes for BuiltinEndpointQos {
    fn number_of_bytes(&self) -> usize {
        4
    }
}

impl Serialize for SpdpDiscoveredParticipantData<Vec<Locator>> {
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

#[cfg(test)]
mod tests {
    use rust_rtps_pim::{
        discovery::types::{BuiltinEndpointQos, BuiltinEndpointSet},
        messages::types::Count,
        structure::types::{Locator, ProtocolVersion},
    };

    use crate::serialize::to_bytes_le;

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
}
