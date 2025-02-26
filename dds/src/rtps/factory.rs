use crate::transport::{
    factory::TransportParticipantFactory, participant::TransportParticipant, types::GuidPrefix,
};

use super::transport::RtpsTransport;

pub struct RtpsParticipantFactoryBuilder {
    interface_name: Option<String>,
    fragment_size: usize,
    udp_receive_buffer_size: Option<usize>,
}

impl Default for RtpsParticipantFactoryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RtpsParticipantFactoryBuilder {
    /// Construct a transport factory builder with all the default options.
    pub fn new() -> Self {
        Self {
            interface_name: None,
            fragment_size: 1344,
            udp_receive_buffer_size: None,
        }
    }

    /// Set the network interface name to use for discovery
    pub fn interface_name(mut self, interface_name: Option<String>) -> Self {
        self.interface_name = interface_name;
        self
    }

    /// Set the maximum size for the data fragments. Types with serialized data above this size will be transmitted as fragments.
    pub fn fragment_size(mut self, fragment_size: usize) -> Self {
        self.fragment_size = fragment_size;
        self
    }

    /// Set the value of the SO_RCVBUF option on the UDP socket. [`None`] corresponds to the OS default
    pub fn udp_receive_buffer_size(mut self, udp_receive_buffer_size: Option<usize>) -> Self {
        self.udp_receive_buffer_size = udp_receive_buffer_size;
        self
    }

    /// Build a new participant factory
    pub fn build(self) -> Result<RtpsParticipantFactory, String> {
        let fragment_size_range = 8..=65000;
        if !fragment_size_range.contains(&self.fragment_size) {
            Err(format!(
                "Interface size out of range. Value must be between in {:?}",
                fragment_size_range
            ))
        } else {
            Ok(RtpsParticipantFactory {
                interface_name: self.interface_name,
                fragment_size: self.fragment_size,
                udp_receive_buffer_size: self.udp_receive_buffer_size,
            })
        }
    }
}

pub struct RtpsParticipantFactory {
    interface_name: Option<String>,
    fragment_size: usize,
    udp_receive_buffer_size: Option<usize>,
}

impl Default for RtpsParticipantFactory {
    fn default() -> Self {
        RtpsParticipantFactoryBuilder::new()
            .build()
            .expect("Default configuration should work")
    }
}

impl TransportParticipantFactory for RtpsParticipantFactory {
    fn create_participant(
        &self,
        guid_prefix: GuidPrefix,
        domain_id: i32,
    ) -> Box<dyn TransportParticipant> {
        Box::new(
            RtpsTransport::new(
                guid_prefix,
                domain_id,
                &self.interface_name,
                self.udp_receive_buffer_size,
                self.fragment_size,
            )
            .unwrap(),
        )
    }
}
