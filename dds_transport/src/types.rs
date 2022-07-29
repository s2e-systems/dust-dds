/// Locator_t
/// Type used to represent the addressing information needed to send a message to an RTPS Endpoint using one of the supported transports.
/// Should be able to hold a discriminator identifying the kind of transport, an address, and a port number. It must be possible to represent the discriminator and port number using 4 octets each, the address using 16 octets.
/// The following values are reserved by the protocol: LOCATOR_INVALID LOCATOR_KIND_INVALID LOCATOR_KIND_RESERVED LOCATOR_KIND_UDPv4 LOCATOR_KIND_UDPv6 LOCATOR_ADDRESS_INVALID LOCATOR_PORT_INVALID
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Locator {
    pub kind: LocatorKind,
    pub port: LocatorPort,
    pub address: LocatorAddress,
}
type LocatorKind = i32;
type LocatorPort = u32;
type LocatorAddress = [u8; 16];

pub const LOCATOR_KIND_INVALID: LocatorKind = -1;
pub const LOCATOR_KIND_RESERVED: LocatorKind = 0;
#[allow(non_upper_case_globals)]
pub const LOCATOR_KIND_UDPv4: LocatorKind = 1;
#[allow(non_upper_case_globals)]
pub const LOCATOR_KIND_UDPv6: LocatorKind = 2;
pub const LOCATOR_PORT_INVALID: LocatorPort = 0;
pub const LOCATOR_ADDRESS_INVALID: LocatorAddress = [0; 16];

pub const LOCATOR_INVALID: Locator = Locator {
    kind: LOCATOR_KIND_INVALID,
    port: LOCATOR_PORT_INVALID,
    address: LOCATOR_ADDRESS_INVALID,
};

impl Locator {
    pub fn new(kind: LocatorKind, port: LocatorPort, address: LocatorAddress) -> Self {
        Self {
            kind,
            port,
            address,
        }
    }
    pub fn kind(&self) -> &LocatorKind {
        &self.kind
    }
    pub fn port(&self) -> &LocatorPort {
        &self.port
    }
    pub fn address(&self) -> &LocatorAddress {
        &self.address
    }
}
