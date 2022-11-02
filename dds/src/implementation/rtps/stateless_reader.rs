use crate::infrastructure::qos_policy::ReliabilityQosPolicyKind;

use super::reader::RtpsReader;

pub struct RtpsStatelessReader(RtpsReader);

impl RtpsStatelessReader {
    pub fn new(reader: RtpsReader) -> Self {
        if reader.get_qos().reliability.kind == ReliabilityQosPolicyKind::Reliable {
            panic!("Reliable stateless reader is not supported");
        }

        Self(reader)
    }

    pub fn reader(&self) -> &RtpsReader {
        &self.0
    }

    pub fn reader_mut(&mut self) -> &mut RtpsReader {
        &mut self.0
    }
}
