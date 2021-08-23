use std::io::Write;
use std::marker::PhantomData;

use crate::serialize::{NumberOfBytes, Serialize};

use crate::submessage_elements::{CountUdp, LocatorUdp, ProtocolVersionUdp, VendorIdUdp};
use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use rust_rtps_pim::{
    behavior::types::Duration,
    discovery::{
        spdp::spdp_discovered_participant_data::SPDPdiscoveredParticipantData,
        types::{BuiltinEndpointQos, BuiltinEndpointSet, DomainId},
    },
    messages::{
        submessage_elements::{
            CountSubmessageElementType, ProtocolVersionSubmessageElementType,
            VendorIdSubmessageElementType,
        },
        types::Count,
    },
    structure::types::{GuidPrefix, Locator, ProtocolVersion, VendorId, Guid},
};

use super::parameterid_list::{
    PID_BUILTIN_ENDPOINT_QOS, PID_BUILTIN_ENDPOINT_SET, PID_DEFAULT_MULTICAST_LOCATOR,
    PID_DEFAULT_UNICAST_LOCATOR, PID_DOMAIN_ID, PID_DOMAIN_TAG, PID_EXPECTS_INLINE_QOS,
    PID_METATRAFFIC_MULTICAST_LOCATOR, PID_METATRAFFIC_UNICAST_LOCATOR, PID_PARTICIPANT_GUID,
    PID_PARTICIPANT_LEASE_DURATION, PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, PID_PROTOCOL_VERSION,
    PID_SENTINEL, PID_VENDORID,
};
use super::types::{DurationUdp, GuidUdp};

#[derive(PartialEq, Debug)]
struct ParticipantProxy {
    domain_id: u32,
    domain_tag: String,
    protocol_version: ProtocolVersionUdp,
    guid: GuidUdp,
    vendor_id: VendorIdUdp,
    expects_inline_qos: bool,
    metatraffic_unicast_locator_list: Vec<LocatorUdp>,
    metatraffic_multicast_locator_list: Vec<LocatorUdp>,
    default_unicast_locator_list: Vec<LocatorUdp>,
    default_multicast_locator_list: Vec<LocatorUdp>,
    available_builtin_endpoints: u32,
    manual_liveliness_count: CountUdp,
    builtin_endpoint_qos: u32,
}

#[derive(PartialEq, Debug)]
pub struct SPDPdiscoveredParticipantDataUdp {
    // ddsParticipantData: DDS::ParticipantBuiltinTopicData,
    participant_proxy: ParticipantProxy,
    lease_duration: DurationUdp,
}

impl SPDPdiscoveredParticipantDataUdp {
    // Constant value from Table 9.14 - ParameterId mapping and default values
    const DEFAULT_DOMAIN_TAG: String = String::new();
    const DEFAULT_EXPECTS_INLINE_QOS: bool = false;
    const DEFAULT_BUILTIN_ENDPOINT_QOS: u32 = 0;
    const DEFAULT_PARTICIPANT_LEASE_DURATION: DurationUdp = DurationUdp {
        seconds: 100,
        fraction: 0,
    };

    pub fn new(
        domain_id: &DomainId,
        domain_tag: &str,
        protocol_version: &ProtocolVersion,
        guid: &Guid,
        vendor_id: &VendorId,
        expects_inline_qos: &bool,
        metatraffic_unicast_locator_list: &[Locator],
        metatraffic_multicast_locator_list: &[Locator],
        default_unicast_locator_list: &[Locator],
        default_multicast_locator_list: &[Locator],
        available_builtin_endpoints: &BuiltinEndpointSet,
        manual_liveliness_count: &Count,
        builtin_endpoint_qos: &BuiltinEndpointQos,
        lease_duration: &Duration,
    ) -> Self {
        Self {
            participant_proxy: ParticipantProxy {
                domain_id: *domain_id,
                domain_tag: domain_tag.to_owned(),
                protocol_version: ProtocolVersionUdp::new(protocol_version),
                guid: (*guid).into(),
                vendor_id: VendorIdUdp::new(vendor_id),
                expects_inline_qos: *expects_inline_qos,
                metatraffic_unicast_locator_list: metatraffic_unicast_locator_list
                    .iter()
                    .map(|x| LocatorUdp::new(x))
                    .collect(),
                metatraffic_multicast_locator_list: metatraffic_multicast_locator_list
                    .iter()
                    .map(|x| LocatorUdp::new(x))
                    .collect(),
                default_unicast_locator_list: default_unicast_locator_list
                    .iter()
                    .map(|x| LocatorUdp::new(x))
                    .collect(),
                default_multicast_locator_list: default_multicast_locator_list
                    .iter()
                    .map(|x| LocatorUdp::new(x))
                    .collect(),
                available_builtin_endpoints: available_builtin_endpoints.0,
                manual_liveliness_count: CountUdp::new(manual_liveliness_count),
                builtin_endpoint_qos: builtin_endpoint_qos.0,
            },
            lease_duration: (*lease_duration).into(),
        }
    }
}

impl SPDPdiscoveredParticipantData for SPDPdiscoveredParticipantDataUdp {
    type LocatorListType = Vec<Locator>;

    fn domain_id(&self) -> DomainId {
        self.participant_proxy.domain_id
    }

    fn domain_tag(&self) -> &str {
        &self.participant_proxy.domain_tag
    }

    fn protocol_version(&self) -> ProtocolVersion {
        self.participant_proxy.protocol_version.value()
    }

    fn guid_prefix(&self) -> GuidPrefix {
        self.participant_proxy.guid.prefix().into()
    }

    fn vendor_id(&self) -> VendorId {
        self.participant_proxy.vendor_id.value()
    }

    fn expects_inline_qos(&self) -> bool {
        self.participant_proxy.expects_inline_qos
    }

    fn metatraffic_unicast_locator_list(&self) -> Self::LocatorListType {
        self.participant_proxy
            .metatraffic_unicast_locator_list
            .iter()
            .map(|x| x.value())
            .collect()
    }

    fn metatraffic_multicast_locator_list(&self) -> Self::LocatorListType {
        self.participant_proxy
            .metatraffic_multicast_locator_list
            .iter()
            .map(|x| x.value())
            .collect()
    }

    fn default_unicast_locator_list(&self) -> Self::LocatorListType {
        self.participant_proxy
            .default_unicast_locator_list
            .iter()
            .map(|x| x.value())
            .collect()
    }

    fn default_multicast_locator_list(&self) -> Self::LocatorListType {
        self.participant_proxy
            .default_multicast_locator_list
            .iter()
            .map(|x| x.value())
            .collect()
    }

    fn available_builtin_endpoints(&self) -> BuiltinEndpointSet {
        BuiltinEndpointSet(self.participant_proxy.available_builtin_endpoints)
    }

    fn manual_liveliness_count(&self) -> Count {
        self.participant_proxy.manual_liveliness_count.value()
    }

    fn builtin_endpoint_qos(&self) -> BuiltinEndpointQos {
        BuiltinEndpointQos(self.participant_proxy.builtin_endpoint_qos)
    }
}

const PL_CDR_LE: [u8; 4] = [0x00, 0x03, 0x00, 0x00];

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
        self.writer.write_u16::<B>(parameter_id)?;
        let length = parameter.number_of_bytes();
        let padding: &[u8] = match length % 4 {
            1 => &[0; 3],
            2 => &[0; 2],
            3 => &[0; 1],
            _ => &[],
        };
        let length_with_padding = (length + padding.len()) as u16;
        self.writer.write_u16::<B>(length_with_padding)?;
        parameter.serialize::<_, B>(&mut self.writer)?;
        padding.serialize::<_, B>(&mut self.writer)
    }
}

impl crate::serialize::Serialize for SPDPdiscoveredParticipantDataUdp {
    fn serialize<W: std::io::Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> crate::serialize::Result {
        writer.write(&PL_CDR_LE)?;

        let mut ser = ParameterSerializer::<_, LittleEndian>::new(&mut writer);
        ser.serialize(PID_DOMAIN_ID, &self.participant_proxy.domain_id)?;
        if &self.participant_proxy.domain_tag != &Self::DEFAULT_DOMAIN_TAG {
            ser.serialize(PID_DOMAIN_TAG, &self.participant_proxy.domain_tag)?;
        }
        ser.serialize(
            PID_PROTOCOL_VERSION,
            &self.participant_proxy.protocol_version,
        )?;
        ser.serialize(PID_PARTICIPANT_GUID, &self.participant_proxy.guid)?;
        ser.serialize(PID_VENDORID, &self.participant_proxy.vendor_id)?;
        if &self.participant_proxy.expects_inline_qos != &Self::DEFAULT_EXPECTS_INLINE_QOS {
            ser.serialize(
                PID_EXPECTS_INLINE_QOS,
                &self.participant_proxy.expects_inline_qos,
            )?;
        }
        ser.serialize_list(
            PID_METATRAFFIC_UNICAST_LOCATOR,
            self.participant_proxy
                .metatraffic_unicast_locator_list
                .as_slice(),
        )?;
        ser.serialize_list(
            PID_METATRAFFIC_MULTICAST_LOCATOR,
            self.participant_proxy
                .metatraffic_multicast_locator_list
                .as_slice(),
        )?;
        ser.serialize_list(
            PID_DEFAULT_UNICAST_LOCATOR,
            self.participant_proxy
                .default_unicast_locator_list
                .as_slice(),
        )?;
        ser.serialize_list(
            PID_DEFAULT_MULTICAST_LOCATOR,
            self.participant_proxy
                .default_multicast_locator_list
                .as_slice(),
        )?;
        ser.serialize(
            PID_BUILTIN_ENDPOINT_SET,
            &self.participant_proxy.available_builtin_endpoints,
        )?;
        ser.serialize(
            PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT,
            &self.participant_proxy.manual_liveliness_count,
        )?;
        if &self.participant_proxy.builtin_endpoint_qos != &Self::DEFAULT_BUILTIN_ENDPOINT_QOS {
            ser.serialize(
                PID_BUILTIN_ENDPOINT_QOS,
                &self.participant_proxy.builtin_endpoint_qos,
            )?;
        }
        if &self.lease_duration != &Self::DEFAULT_PARTICIPANT_LEASE_DURATION {
            ser.serialize(PID_PARTICIPANT_LEASE_DURATION, &self.lease_duration)?;
        }
        ser.serialize(PID_SENTINEL, &[])?;
        Ok(())
    }
}

impl<'de> crate::deserialize::Deserialize<'de> for SPDPdiscoveredParticipantDataUdp {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        // let _representation: [u8; 4] = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        // let parameter_list: () = ();
        //     // crate::deserialize::Deserialize::deserialize::<B>(buf)?;

        // let domain_id = parameter_list
        //     .get(PID_DOMAIN_ID)
        //     .ok_or(std::io::Error::new(
        //         std::io::ErrorKind::Other,
        //         "Missing PID_DOMAIN_ID parameter",
        //     ))?;

        // let domain_tag = parameter_list
        //     .get(PID_DOMAIN_TAG)
        //     .unwrap_or(Self::DEFAULT_DOMAIN_TAG);

        // let protocol_version: ProtocolVersionUdp =
        //     parameter_list
        //         .get(PID_PROTOCOL_VERSION)
        //         .ok_or(std::io::Error::new(
        //             std::io::ErrorKind::Other,
        //             "Missing PID_PROTOCOL_VERSION parameter",
        //         ))?;

        // let guid: GuidUdp = parameter_list
        //     .get(PID_PARTICIPANT_GUID)
        //     .ok_or(std::io::Error::new(
        //         std::io::ErrorKind::Other,
        //         "Missing PID_PARTICIPANT_GUID parameter",
        //     ))?;

        // let vendor_id: VendorIdUdp = parameter_list.get(PID_VENDORID).ok_or(
        //     std::io::Error::new(std::io::ErrorKind::Other, "Missing PID_VENDORID parameter"),
        // )?;

        // let expects_inline_qos = parameter_list
        //     .get(PID_EXPECTS_INLINE_QOS)
        //     .unwrap_or(Self::DEFAULT_EXPECTS_INLINE_QOS);

        // let metatraffic_unicast_locator_list =
        //     parameter_list.get_list(PID_METATRAFFIC_UNICAST_LOCATOR);

        // let metatraffic_multicast_locator_list =
        //     parameter_list.get_list(PID_METATRAFFIC_MULTICAST_LOCATOR);

        // let default_unicast_locator_list = parameter_list.get_list(PID_DEFAULT_UNICAST_LOCATOR);

        // let default_multicast_locator_list = parameter_list.get_list(PID_DEFAULT_MULTICAST_LOCATOR);

        // let available_builtin_endpoints =
        //     parameter_list
        //         .get(PID_BUILTIN_ENDPOINT_SET)
        //         .ok_or(std::io::Error::new(
        //             std::io::ErrorKind::Other,
        //             "Missing PID_BUILTIN_ENDPOINT_SET parameter",
        //         ))?;

        // let manual_liveliness_count = parameter_list
        //     .get(PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT)
        //     .ok_or(std::io::Error::new(
        //         std::io::ErrorKind::Other,
        //         "Missing PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT parameter",
        //     ))?;

        // let builtin_endpoint_qos = parameter_list
        //     .get(PID_BUILTIN_ENDPOINT_QOS)
        //     .unwrap_or(Self::DEFAULT_BUILTIN_ENDPOINT_QOS);

        // let lease_duration = parameter_list
        //     .get(PID_PARTICIPANT_LEASE_DURATION)
        //     .unwrap_or(Self::DEFAULT_PARTICIPANT_LEASE_DURATION);

        // let participant_proxy = ParticipantProxy {
        //     domain_id,
        //     domain_tag,
        //     protocol_version,
        //     guid,
        //     vendor_id,
        //     expects_inline_qos,
        //     metatraffic_unicast_locator_list,
        //     metatraffic_multicast_locator_list,
        //     default_unicast_locator_list,
        //     default_multicast_locator_list,
        //     available_builtin_endpoints,
        //     manual_liveliness_count,
        //     builtin_endpoint_qos,
        // };

        // Ok(Self {
        //     participant_proxy: participant_proxy,
        //     lease_duration: lease_duration,
        // })
        todo!()
    }
}
#[cfg(test)]
mod tests {
    use crate::{deserialize::from_bytes_le, serialize::to_bytes_le};

    use super::*;
    use rust_rtps_pim::structure::types::ENTITYID_PARTICIPANT;

    #[test]
    pub fn serialize_spdp_default_data() {
        let domain_id = 1;
        let domain_tag = &SPDPdiscoveredParticipantDataUdp::DEFAULT_DOMAIN_TAG;
        let protocol_version = ProtocolVersion { major: 2, minor: 4 };
        let guid = Guid::new([1; 12], ENTITYID_PARTICIPANT);
        let vendor_id = [9, 9];
        let expects_inline_qos = SPDPdiscoveredParticipantDataUdp::DEFAULT_EXPECTS_INLINE_QOS;
        let metatraffic_unicast_locator_list = &[];
        let metatraffic_multicast_locator_list = &[];
        let default_unicast_locator_list = &[];
        let default_multicast_locator_list = &[];
        let available_builtin_endpoints =
            BuiltinEndpointSet::new(BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR);
        let manual_liveliness_count = Count(2);
        let builtin_endpoint_qos = BuiltinEndpointQos::new(0);
        let lease_duration =
            SPDPdiscoveredParticipantDataUdp::DEFAULT_PARTICIPANT_LEASE_DURATION.into();

        let spdp_discovered_participant_data = SPDPdiscoveredParticipantDataUdp::new(
            &domain_id,
            domain_tag,
            &protocol_version,
            &guid,
            &vendor_id,
            &expects_inline_qos,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            default_unicast_locator_list,
            default_multicast_locator_list,
            &available_builtin_endpoints,
            &manual_liveliness_count,
            &builtin_endpoint_qos,
            &lease_duration,
        );

        let serialized_data = to_bytes_le(&spdp_discovered_participant_data).unwrap();
        let expected_data = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x0f, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
            0x01, 0x00, 0x00, 0x00, // DomainId(1)
            0x15, 0x00, 0x04, 0x00, // PID_PROTOCOL_VERSION, Length: 4
            0x02, 0x04, 0x00, 0x00, // ProtocolVersion{major:2, minor:4}
            0x50, 0x00, 0x10, 0x00, // PID_PARTICIPANT_GUID, Length: 16
            0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
            0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
            0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
            0x00, 0x00, 0x01, 0xc1, // EntityId(ENTITYID_PARTICIPANT)
            0x16, 0x00, 0x04, 0x00, // PID_VENDORID, Length:4,
            0x09, 0x09, 0x00, 0x00, // VendorId([9,9])
            0x58, 0x00, 0x04, 0x00, // PID_BUILTIN_ENDPOINT_SET, Length: 4
            0x02, 0x00, 0x00, 0x00, // BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
            0x34, 0x00, 0x04, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Length: 4
            0x02, 0x00, 0x00, 0x00, // Count(2)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length: 0
        ];

        assert_eq!(serialized_data, expected_data);
    }

    #[test]
    pub fn serialize_complete_spdp_discovered_participant_data() {
        let locator1 = Locator::new(1, 1, [1; 16]);
        let locator2 = Locator::new(2, 2, [2; 16]);

        let domain_id = 1;
        let domain_tag = "abc";
        let protocol_version = ProtocolVersion { major: 2, minor: 4 };
        let guid = Guid::new([1; 12], ENTITYID_PARTICIPANT);
        let vendor_id = [9, 9];
        let expects_inline_qos = true;
        let metatraffic_unicast_locator_list = &[locator1, locator2];
        let metatraffic_multicast_locator_list = &[locator1, locator2];
        let default_unicast_locator_list = &[locator1, locator2];
        let default_multicast_locator_list = &[locator1, locator2];
        let available_builtin_endpoints =
            BuiltinEndpointSet::new(BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR);
        let manual_liveliness_count = Count(2);
        let builtin_endpoint_qos = BuiltinEndpointQos::new(
            BuiltinEndpointQos::BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER,
        );
        let lease_duration = Duration {
            seconds: 10,
            fraction: 0,
        };

        let spdp_discovered_participant_data = SPDPdiscoveredParticipantDataUdp::new(
            &domain_id,
            domain_tag,
            &protocol_version,
            &guid,
            &vendor_id,
            &expects_inline_qos,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            default_unicast_locator_list,
            default_multicast_locator_list,
            &available_builtin_endpoints,
            &manual_liveliness_count,
            &builtin_endpoint_qos,
            &lease_duration,
        );

        let serialized_data = to_bytes_le(&spdp_discovered_participant_data).unwrap();
        let expected_data = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x0f, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
            0x01, 0x00, 0x00, 0x00, // DomainId(1)
            0x14, 0x40, 0x08, 0x00, // PID_DOMAIN_TAG, Length: 8
            0x04, 0x00, 0x00, 0x00, // DomainTag(length: 4)
            b'a', b'b', b'c', 0x00, // DomainTag('abc')
            0x15, 0x00, 0x04, 0x00, // PID_PROTOCOL_VERSION, Length: 4
            0x02, 0x04, 0x00, 0x00, // ProtocolVersion{major:2, minor:4}
            0x50, 0x00, 0x10, 0x00, // PID_PARTICIPANT_GUID, Length: 16
            0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
            0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
            0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
            0x00, 0x00, 0x01, 0xc1, // EntityId(ENTITYID_PARTICIPANT)
            0x16, 0x00, 0x04, 0x00, // PID_VENDORID, Length:4,
            0x09, 0x09, 0x00, 0x00, // VendorId([9,9])
            0x43, 0x00, 0x04, 0x00, // PID_EXPECTS_INLINE_QOS, Length: 4,
            0x01, 0x00, 0x00, 0x00, // True
            0x32, 0x00, 0x18, 0x00, // PID_METATRAFFIC_UNICAST_LOCATOR, Length: 24,
            0x01, 0x00, 0x00, 0x00, // Locator{kind:1
            0x01, 0x00, 0x00, 0x00, // port:1,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address: [1;16]
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            0x32, 0x00, 0x18, 0x00, // PID_METATRAFFIC_UNICAST_LOCATOR, Length: 24,
            0x02, 0x00, 0x00, 0x00, // Locator{kind:2
            0x02, 0x00, 0x00, 0x00, // port:2,
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // address: [2;16]
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // }
            0x33, 0x00, 0x18, 0x00, // PID_METATRAFFIC_MULTICAST_LOCATOR, Length: 24,
            0x01, 0x00, 0x00, 0x00, // Locator{kind:1
            0x01, 0x00, 0x00, 0x00, // port:1,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address: [1;16]
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            0x33, 0x00, 0x18, 0x00, // PID_METATRAFFIC_MULTICAST_LOCATOR, Length: 24,
            0x02, 0x00, 0x00, 0x00, // Locator{kind:2
            0x02, 0x00, 0x00, 0x00, // port:2,
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // address: [2;16]
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // }
            0x31, 0x00, 0x18, 0x00, // PID_DEFAULT_UNICAST_LOCATOR, Length: 24,
            0x01, 0x00, 0x00, 0x00, // Locator{kind:1
            0x01, 0x00, 0x00, 0x00, // port:1,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address: [1;16]
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            0x31, 0x00, 0x18, 0x00, // PID_DEFAULT_UNICAST_LOCATOR, Length: 24,
            0x02, 0x00, 0x00, 0x00, // Locator{kind:2
            0x02, 0x00, 0x00, 0x00, // port:2,
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // address: [2;16]
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // }
            0x48, 0x00, 0x18, 0x00, // PID_DEFAULT_MULTICAST_LOCATOR, Length: 24,
            0x01, 0x00, 0x00, 0x00, // Locator{kind:1
            0x01, 0x00, 0x00, 0x00, // port:1,
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // address: [1;16]
            0x01, 0x01, 0x01, 0x01, //
            0x01, 0x01, 0x01, 0x01, // }
            0x48, 0x00, 0x18, 0x00, // PID_DEFAULT_MULTICAST_LOCATOR, Length: 24,
            0x02, 0x00, 0x00, 0x00, // Locator{kind:2
            0x02, 0x00, 0x00, 0x00, // port:2,
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // address: [2;16]
            0x02, 0x02, 0x02, 0x02, //
            0x02, 0x02, 0x02, 0x02, // }
            0x58, 0x00, 0x04, 0x00, // PID_BUILTIN_ENDPOINT_SET, Length: 4
            0x02, 0x00, 0x00, 0x00, // BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
            0x34, 0x00, 0x04, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Length: 4
            0x02, 0x00, 0x00, 0x00, // Count(2)
            0x77, 0x00, 0x04, 0x00, // PID_BUILTIN_ENDPOINT_QOS, Length: 4
            0x00, 0x00, 0x00, 0x20, // BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER
            0x02, 0x00, 0x08, 0x00, // PID_PARTICIPANT_LEASE_DURATION, Length: 8
            0x0a, 0x00, 0x00, 0x00, // Duration{seconds:30,
            0x00, 0x00, 0x00, 0x00, //          fraction:0}
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length: 0
        ];

        assert_eq!(serialized_data, expected_data);
    }

    #[test]
    fn deserialize_complete_spdp_discovered_participant_data() {
        #[rustfmt::skip]
        let spdp_discovered_participant_data: SPDPdiscoveredParticipantDataUdp = from_bytes_le(&[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x0f, 0x00, 4, 0x00,    // PID_DOMAIN_ID, Length
            0x01, 0x00, 0x00, 0x00, // DomainId
            0x14, 0x40, 8, 0x00,    // PID_DOMAIN_TAG, Length
            4,    0,    0,    0,             // Length: 4
            b'a', b'b', b'c', 0x00, // DomainTag
            0x15, 0x00, 4, 0x00,    // PID_PROTOCOL_VERSION, Length
            0x02, 0x04, 0x00, 0x00, // ProtocolVersion: major, minor
            0x50, 0x00, 16, 0x00,   // PID_PARTICIPANT_GUID, Length
            0x01, 0x01, 0x01, 0x01, // GuidPrefix
            0x01, 0x01, 0x01, 0x01, // GuidPrefix
            0x01, 0x01, 0x01, 0x01, // GuidPrefix
            0x00, 0x00, 0x01, 0xc1, // EntityId(ENTITYID_PARTICIPANT)
            0x16, 0x00, 4, 0x00,    // PID_VENDORID, Length
            0x09, 0x09, 0x00, 0x00, // VendorId
            0x43, 0x00, 4, 0x00,    // PID_EXPECTS_INLINE_QOS, Length
            0x01, 0x00, 0x00, 0x00, // True
            0x32, 0x00, 24, 0x00,   // PID_METATRAFFIC_UNICAST_LOCATOR, Length
            0x01, 0x00, 0x00, 0x00, // Locator: kind
            0x01, 0x00, 0x00, 0x00, // Locator: port
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x32, 0x00, 24, 0x00,   // PID_METATRAFFIC_UNICAST_LOCATOR, Length
            0x02, 0x00, 0x00, 0x00, // Locator: kind
            0x02, 0x00, 0x00, 0x00, // Locator: port
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x33, 0x00, 24, 0x00,   // PID_METATRAFFIC_MULTICAST_LOCATOR, Length
            0x01, 0x00, 0x00, 0x00, // Locator: kind
            0x01, 0x00, 0x00, 0x00, // Locator: port
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x33, 0x00, 24, 0x00,   // PID_METATRAFFIC_MULTICAST_LOCATOR, Length
            0x02, 0x00, 0x00, 0x00, // Locator: kind
            0x02, 0x00, 0x00, 0x00, // Locator: port,
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x31, 0x00, 24, 0x00,   // PID_DEFAULT_UNICAST_LOCATOR, Length
            0x01, 0x00, 0x00, 0x00, // Locator: kind
            0x01, 0x00, 0x00, 0x00, // Locator: port
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x31, 0x00, 24, 0x00,   // PID_DEFAULT_UNICAST_LOCATOR, Length
            0x02, 0x00, 0x00, 0x00, // Locator: kind
            0x02, 0x00, 0x00, 0x00, // Locator: port
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x48, 0x00, 24, 0x00,   // PID_DEFAULT_MULTICAST_LOCATOR, Length
            0x01, 0x00, 0x00, 0x00, // Locator: kind
            0x01, 0x00, 0x00, 0x00, // Locator: port
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x01, 0x01, 0x01, 0x01, // Locator: address
            0x48, 0x00, 024, 0x00, // PID_DEFAULT_MULTICAST_LOCATOR, Length,
            0x02, 0x00, 0x00, 0x00, // Locator: kind
            0x02, 0x00, 0x00, 0x00, // Locator: port
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x02, 0x02, 0x02, 0x02, // Locator: address
            0x58, 0x00, 4, 0x00,    // PID_BUILTIN_ENDPOINT_SET, Length
            0x02, 0x00, 0x00, 0x00, // BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
            0x34, 0x00, 4, 0x00,    // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Length
            0x02, 0x00, 0x00, 0x00, // Count
            0x77, 0x00, 4, 0x00,    // PID_BUILTIN_ENDPOINT_QOS, Length: 4
            0x00, 0x00, 0x00, 0x20, // BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER
            0x02, 0x00, 8, 0x00,    // PID_PARTICIPANT_LEASE_DURATION, Length
            10, 0x00, 0x00, 0x00,   // Duration: seconds
            0x00, 0x00, 0x00, 0x00, // Duration: fraction
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length
        ]).unwrap();

        let locator1 = Locator::new(1, 1, [1; 16]);
        let locator2 = Locator::new(2, 2, [2; 16]);

        let domain_id = 1;
        let domain_tag = "abc";
        let protocol_version = ProtocolVersion { major: 2, minor: 4 };
        let guid = Guid::new([1; 12], ENTITYID_PARTICIPANT);
        let vendor_id = [9, 9];
        let expects_inline_qos = true;
        let metatraffic_unicast_locator_list = &[locator1, locator2];
        let metatraffic_multicast_locator_list = &[locator1, locator2];
        let default_unicast_locator_list = &[locator1, locator2];
        let default_multicast_locator_list = &[locator1, locator2];
        let available_builtin_endpoints =
            BuiltinEndpointSet::new(BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR);
        let manual_liveliness_count = Count(2);
        let builtin_endpoint_qos = BuiltinEndpointQos::new(
            BuiltinEndpointQos::BEST_EFFORT_PARTICIPANT_MESSAGE_DATA_READER,
        );
        let lease_duration = Duration {
            seconds: 10,
            fraction: 0,
        };

        let expected_spdp_discovered_participant_data = SPDPdiscoveredParticipantDataUdp::new(
            &domain_id,
            domain_tag,
            &protocol_version,
            &guid,
            &vendor_id,
            &expects_inline_qos,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            default_unicast_locator_list,
            default_multicast_locator_list,
            &available_builtin_endpoints,
            &manual_liveliness_count,
            &builtin_endpoint_qos,
            &lease_duration,
        );

        assert_eq!(
            spdp_discovered_participant_data,
            expected_spdp_discovered_participant_data
        );
    }

    #[test]
    fn deserialize_default_spdp_discovered_participant_data() {
        #[rustfmt::skip]
        let spdp_discovered_participant_data: SPDPdiscoveredParticipantDataUdp = from_bytes_le(&[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x0f, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
            0x01, 0x00, 0x00, 0x00, // DomainId(1)
            0x15, 0x00, 0x04, 0x00, // PID_PROTOCOL_VERSION, Length: 4
            0x02, 0x04, 0x00, 0x00, // ProtocolVersion{major:2, minor:4}
            0x50, 0x00, 0x10, 0x00, // PID_PARTICIPANT_GUID, Length: 16
            0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
            0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
            0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
            0x00, 0x00, 0x01, 0xc1, // EntityId(ENTITYID_PARTICIPANT)
            0x16, 0x00, 0x04, 0x00, // PID_VENDORID, Length:4,
            0x09, 0x09, 0x00, 0x00, // VendorId([9,9])
            0x58, 0x00, 0x04, 0x00, // PID_BUILTIN_ENDPOINT_SET, Length: 4
            0x02, 0x00, 0x00, 0x00, // BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
            0x34, 0x00, 0x04, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Length: 4
            0x02, 0x00, 0x00, 0x00, // Count(2)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length: 0
        ]).unwrap();

        let domain_id = 1;
        let domain_tag = &SPDPdiscoveredParticipantDataUdp::DEFAULT_DOMAIN_TAG;
        let protocol_version = ProtocolVersion { major: 2, minor: 4 };
        let guid = Guid::new([1; 12], ENTITYID_PARTICIPANT);
        let vendor_id = [9, 9];
        let expects_inline_qos = SPDPdiscoveredParticipantDataUdp::DEFAULT_EXPECTS_INLINE_QOS;
        let metatraffic_unicast_locator_list = &[];
        let metatraffic_multicast_locator_list = &[];
        let default_unicast_locator_list = &[];
        let default_multicast_locator_list = &[];
        let available_builtin_endpoints =
            BuiltinEndpointSet::new(BuiltinEndpointSet::BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR);
        let manual_liveliness_count = Count(2);
        let builtin_endpoint_qos = BuiltinEndpointQos::new(0);
        let lease_duration =
            SPDPdiscoveredParticipantDataUdp::DEFAULT_PARTICIPANT_LEASE_DURATION.into();

        let expected_spdp_discovered_participant_data = SPDPdiscoveredParticipantDataUdp::new(
            &domain_id,
            domain_tag,
            &protocol_version,
            &guid,
            &vendor_id,
            &expects_inline_qos,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            default_unicast_locator_list,
            default_multicast_locator_list,
            &available_builtin_endpoints,
            &manual_liveliness_count,
            &builtin_endpoint_qos,
            &lease_duration,
        );

        assert_eq!(
            spdp_discovered_participant_data,
            expected_spdp_discovered_participant_data
        );
    }

    #[test]
    fn deserialize_wrong_spdp_discovered_participant_data() {
        #[rustfmt::skip]
        let result: std::result::Result<SPDPdiscoveredParticipantDataUdp, _> = from_bytes_le(&[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x0f, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
            0x01, 0x00, 0x00, 0x00, // DomainId(1)
            0x50, 0x00, 0x10, 0x00, // PID_PARTICIPANT_GUID, Length: 16
            0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
            0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
            0x01, 0x01, 0x01, 0x01, // GuidPrefix([1;12])
            0x00, 0x00, 0x01, 0xc1, // EntityId(ENTITYID_PARTICIPANT)
            0x16, 0x00, 0x04, 0x00, // PID_VENDORID, Length:4,
            0x09, 0x09, 0x00, 0x00, // VendorId([9,9])
            0x58, 0x00, 0x04, 0x00, // PID_BUILTIN_ENDPOINT_SET, Length: 4
            0x02, 0x00, 0x00, 0x00, // BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
            0x34, 0x00, 0x04, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Length: 4
            0x02, 0x00, 0x00, 0x00, // Count(2)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length: 0
        ]);
        match result {
            Err(err) => {
                assert_eq!(err.kind(), std::io::ErrorKind::Other);
                assert_eq!(err.to_string(), "Missing PID_PROTOCOL_VERSION parameter")
            }
            _ => panic!(),
        }

        #[rustfmt::skip]
        let result: Result<SPDPdiscoveredParticipantDataUdp, _> = from_bytes_le(&[
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE
            0x0f, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
            0x01, 0x00, 0x00, 0x00, // DomainId(1)
            0x15, 0x00, 0x04, 0x00, // PID_PROTOCOL_VERSION, Length: 4
            0x02, 0x04, 0x00, 0x00, // ProtocolVersion{major:2, minor:4}
            0x16, 0x00, 0x04, 0x00, // PID_VENDORID, Length:4,
            0x09, 0x09, 0x00, 0x00, // VendorId([9,9])
            0x58, 0x00, 0x04, 0x00, // PID_BUILTIN_ENDPOINT_SET, Length: 4
            0x02, 0x00, 0x00, 0x00, // BUILTIN_ENDPOINT_PARTICIPANT_DETECTOR
            0x34, 0x00, 0x04, 0x00, // PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT, Length: 4
            0x02, 0x00, 0x00, 0x00, // Count(2)
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, Length: 0
        ]);

        match result {
            Err(err) => {
                assert_eq!(err.kind(), std::io::ErrorKind::Other);
                assert_eq!(err.to_string(), "Missing PID_PARTICIPANT_GUID parameter")
            }
            _ => panic!(),
        }
    }
}
