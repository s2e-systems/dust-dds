use dust_dds::{
    infrastructure::instance::InstanceHandle,
    rtps_messages::overall_structure::{RtpsMessageRead, RtpsSubmessageReadKind},
    transport::types::{GuidPrefix, Locator},
    xtypes::{
        deserialize::XTypesDeserialize,
        error::XTypesError,
        xcdr_deserializer::{Xcdr1BeDeserializer, Xcdr1Deserializer},
    },
};
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use socket2::Socket;
use std::{
    collections::{hash_map::Entry, HashMap},
    io::{BufRead, Read},
    net::{Ipv4Addr, SocketAddr, UdpSocket},
};

type RepresentationIdentifier = [u8; 2];
const PL_CDR_BE: RepresentationIdentifier = [0x00, 0x02];
const PL_CDR_LE: RepresentationIdentifier = [0x00, 0x03];

type ParameterId = u16;
const PID_SENTINEL: ParameterId = 0x0001;
const PID_DEFAULT_UNICAST_LOCATOR: ParameterId = 0x0031;
const PID_DEFAULT_MULTICAST_LOCATOR: ParameterId = 0x0048;
const PID_METATRAFFIC_UNICAST_LOCATOR: ParameterId = 0x0032;
const PID_METATRAFFIC_MULTICAST_LOCATOR: ParameterId = 0x0033;
const PID_PARTICIPANT_GUID: ParameterId = 0x0050;
const PID_DOMAIN_ID: ParameterId = 0x000f;
const PID_DOMAIN_TAG: ParameterId = 0x4014;
const PID_DISCOVERED_PARTICIPANT: ParameterId = 0x8020;

#[derive(Debug, PartialEq, Eq)]
enum Error {
    InvalidData,
}
impl From<std::io::Error> for Error {
    fn from(_value: std::io::Error) -> Self {
        Self::InvalidData
    }
}
impl From<XTypesError> for Error {
    fn from(_value: XTypesError) -> Self {
        Self::InvalidData
    }
}

struct ParameterList(Vec<u8>);

impl ParameterList {
    fn list<T: for<'de> XTypesDeserialize<'de>, const PID: ParameterId>(
        &self,
    ) -> Result<Vec<T>, Error> {
        let reader = &mut &self.0[..];
        let mut current_pid = [0, 0];
        let mut length = [0, 0];
        let mut representation_identifier = [0, 0];
        reader.read(&mut representation_identifier)?;
        reader.consume(2);

        let mut list = Vec::new();
        loop {
            reader.read(&mut current_pid)?;
            reader.read(&mut length)?;

            match representation_identifier {
                PL_CDR_BE => {
                    let current_pid = ParameterId::from_be_bytes(current_pid);
                    if current_pid == PID {
                        list.push(T::deserialize(&mut Xcdr1BeDeserializer::new(reader))?);
                    } else if current_pid == PID_SENTINEL {
                        return Ok(list);
                    } else {
                        reader.consume(u16::from_be_bytes(length) as usize)
                    }
                }
                PL_CDR_LE => {
                    let current_pid = ParameterId::from_le_bytes(current_pid);
                    if current_pid == PID {
                        list.push(T::deserialize(&mut Xcdr1Deserializer::new(reader))?);
                    } else if current_pid == PID_SENTINEL {
                        return Ok(list);
                    } else {
                        reader.consume(u16::from_le_bytes(length) as usize)
                    }
                }
                _ => return Err(Error::InvalidData),
            }
        }
    }

    fn entry<T: for<'de> XTypesDeserialize<'de>, const PID: ParameterId>(
        &self,
    ) -> Result<T, Error> {
        let reader = &mut &self.0[..];
        let mut current_pid = [0, 0];
        let mut length = [0, 0];
        let mut representation_identifier = [0, 0];
        reader.read(&mut representation_identifier)?;
        reader.consume(2);

        loop {
            reader.read(&mut current_pid)?;
            reader.read(&mut length)?;

            match representation_identifier {
                PL_CDR_BE => {
                    let current_pid = ParameterId::from_be_bytes(current_pid);
                    if current_pid == PID {
                        return Ok(T::deserialize(&mut Xcdr1BeDeserializer::new(reader))?);
                    } else if current_pid == PID_SENTINEL {
                        return Err(Error::InvalidData);
                    } else {
                        reader.consume(u16::from_be_bytes(length) as usize)
                    }
                }
                PL_CDR_LE => {
                    let current_pid = ParameterId::from_le_bytes(current_pid);
                    if current_pid == PID {
                        return Ok(T::deserialize(&mut Xcdr1Deserializer::new(reader))?);
                    } else if current_pid == PID_SENTINEL {
                        return Err(Error::InvalidData);
                    } else {
                        reader.consume(u16::from_le_bytes(length) as usize)
                    }
                }
                _ => return Err(Error::InvalidData),
            }
        }
    }

    fn default_unicast_locator_list(&self) -> Result<Vec<Locator>, Error> {
        self.list::<_, PID_DEFAULT_UNICAST_LOCATOR>()
    }

    fn default_multicast_locator_list(&self) -> Result<Vec<Locator>, Error> {
        self.list::<_, PID_DEFAULT_MULTICAST_LOCATOR>()
    }

    fn metatraffic_unicast_locator_list(&self) -> Result<Vec<Locator>, Error> {
        self.list::<_, PID_METATRAFFIC_UNICAST_LOCATOR>()
    }

    fn metatraffic_multicast_locator_list(&self) -> Result<Vec<Locator>, Error> {
        self.list::<_, PID_METATRAFFIC_MULTICAST_LOCATOR>()
    }

    fn discovered_participant_list(&self) -> Result<Vec<InstanceHandle>, Error> {
        self.list::<_, PID_DISCOVERED_PARTICIPANT>()
    }

    fn guid_prefix(&self) -> Result<GuidPrefix, Error> {
        self.entry::<_, PID_PARTICIPANT_GUID>()
    }

    fn domain_id(&self) -> Result<GuidPrefix, Error> {
        self.entry::<_, PID_DOMAIN_ID>()
    }

    fn domain_tag(&self) -> Result<GuidPrefix, Error> {
        self.entry::<_, PID_DOMAIN_TAG>()
    }
}

fn main() {
    let port = 7400;
    let socket_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));

    let socket = Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )
    .unwrap();

    socket.set_reuse_address(true).unwrap();
    socket.set_nonblocking(false).unwrap();
    socket.bind(&socket_addr.into()).unwrap();

    let socket = UdpSocket::from(socket);

    let multicast_addr = Ipv4Addr::new(239, 255, 0, 1);
    let interface_address_list = get_interface_address_list(None);
    for interface_addr in interface_address_list {
        match interface_addr {
            Addr::V4(a) => {
                let r = socket.join_multicast_v4(&multicast_addr, &a.ip);
                if let Err(e) = r {
                    println!(
                        "Failed to join multicast group on address {} with error {}",
                        a.ip, e
                    )
                }
            }
            Addr::V6(_) => (),
        }
    }

    let mut participant_list = HashMap::new();

    let mut buf = [0u8; 65000];
    println!("Started Dust DDS Telescope");
    loop {
        let received_size = socket.recv(&mut buf).unwrap();
        if let Ok(m) = RtpsMessageRead::try_from(&buf[0..received_size]) {
            for submessage in m.submessages() {
                if let RtpsSubmessageReadKind::Data(d) = submessage {
                    let discovered_participant =
                        ParameterList(d.serialized_payload().as_ref().to_vec());

                    match participant_list.entry(discovered_participant.guid_prefix().unwrap()) {
                        Entry::Vacant(e) => {
                            println!(
                                "Discovered participant GUID {:?} on domain {:?} with tag {:?}",
                                discovered_participant.guid_prefix(),
                                discovered_participant.domain_id(),
                                discovered_participant.domain_tag(),
                            );
                            println!(
                                "Participant metattrafic unicast locator list {:?}",
                                discovered_participant.metatraffic_unicast_locator_list()
                            );
                            println!(
                                "Participant metattrafic multicast locator list {:?}",
                                discovered_participant.metatraffic_multicast_locator_list()
                            );
                            println!(
                                "Participant default unicast locator list {:?}",
                                discovered_participant.default_unicast_locator_list()
                            );
                            println!(
                                "Participant default multicast locator list {:?}",
                                discovered_participant.default_multicast_locator_list()
                            );
                            println!(
                                "Discovered participant list {:?}",
                                discovered_participant.discovered_participant_list()
                            );
                            e.insert(discovered_participant);
                            println!("\n\n")
                        }
                        Entry::Occupied(mut e) => {
                            if e.get().discovered_participant_list()
                                != discovered_participant.discovered_participant_list()
                            {
                                println!(
                                    "Updated participant GUID {:?} on domain {:?} with tag {:?}",
                                    discovered_participant.guid_prefix(),
                                    discovered_participant.domain_id(),
                                    discovered_participant.domain_tag(),
                                );
                                println!(
                                    "Discovered participant list {:?}",
                                    discovered_participant.discovered_participant_list()
                                );
                                e.insert(discovered_participant);
                            }
                        }
                    }
                }
            }
        } else {
            println!("Received data not representing an RTPS message");
        }
    }
}

fn get_interface_address_list(interface_name: Option<&String>) -> Vec<Addr> {
    NetworkInterface::show()
        .expect("Could not scan interfaces")
        .into_iter()
        .filter(|x| {
            if let Some(if_name) = interface_name {
                &x.name == if_name
            } else {
                true
            }
        })
        .flat_map(|i| {
            i.addr.into_iter().filter(|a| match a {
                #[rustfmt::skip]
                Addr::V4(v4) if !v4.ip.is_loopback() => true,
                _ => false,
            })
        })
        .collect()
}
