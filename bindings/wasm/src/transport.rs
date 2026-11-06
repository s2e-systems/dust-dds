use dust_dds::transport::{
    interface::{RtpsParticipant, Transport, Write},
    types::{ENTITYID_PARTICIPANT, Guid, Locator},
};

pub struct WasmWriter;

impl Write for WasmWriter {
    fn write(&mut self, _buf: &[u8]) {}
    fn flush(&mut self) {}
}

pub struct WasmRtpsParticipant {
    guid: Guid,
}

impl RtpsParticipant for WasmRtpsParticipant {
    type Writer<'a> = WasmWriter;

    fn guid(&self) -> Guid {
        self.guid
    }

    fn default_unicast_locator_list(&self) -> &[Locator] {
        &[]
    }

    fn metatraffic_unicast_locator_list(&self) -> &[Locator] {
        &[]
    }

    fn metatraffic_multicast_locator_list(&self) -> &[Locator] {
        &[]
    }

    fn default_multicast_locator_list(&self) -> &[Locator] {
        &[]
    }

    fn fragment_size(&self) -> usize {
        1300
    }

    fn writer<'a>(&'a mut self, _locator: &'a [Locator]) -> Self::Writer<'a> {
        WasmWriter
    }

    async fn read(&mut self) -> &[u8] {
        core::future::pending().await
    }
}

pub struct WasmTransport {
    participant_counter: u32,
}

impl WasmTransport {
    pub fn new() -> Self {
        Self {
            participant_counter: 0,
        }
    }
}

impl Default for WasmTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for WasmTransport {
    type Participant = WasmRtpsParticipant;

    fn create_participant(&mut self, _domain_id: i32) -> Self::Participant {
        self.participant_counter = self.participant_counter.wrapping_add(1);
        let c = self.participant_counter.to_le_bytes();
        let guid_prefix = [
            0x01, 0x02, 0x03, 0x04,
            0x05, 0x06, 0x07, 0x08,
            c[0], c[1], c[2], c[3],
        ];
        WasmRtpsParticipant {
            guid: Guid::new(guid_prefix, ENTITYID_PARTICIPANT),
        }
    }

    fn delete_participant(&mut self, _handle: Guid) {}
}
