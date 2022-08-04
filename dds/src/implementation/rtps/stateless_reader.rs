use super::{reader::RtpsReader, types::ReliabilityKind};

pub struct RtpsStatelessReader(RtpsReader);

impl RtpsStatelessReader {
    pub fn new(reader: RtpsReader) -> Self {
        if reader.reliability_level() == ReliabilityKind::Reliable {
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

#[cfg(test)]
mod tests {
    use crate::{
        dcps_psm::Duration,
        implementation::rtps::{
            endpoint::RtpsEndpoint,
            types::{EntityId, Guid, GuidPrefix, TopicKind, USER_DEFINED_READER_NO_KEY},
        },
        infrastructure::qos::DataReaderQos,
    };

    use super::*;

    #[test]
    #[should_panic]
    fn reliable_stateless_data_reader_not_constructible() {
        let endpoint = RtpsEndpoint::new(
            Guid::new(
                GuidPrefix([3; 12]),
                EntityId::new([4, 1, 3], USER_DEFINED_READER_NO_KEY),
            ),
            TopicKind::NoKey,
            ReliabilityKind::Reliable,
            &[],
            &[],
        );
        let reader = RtpsReader::new(
            endpoint,
            Duration::new(0, 0),
            Duration::new(0, 0),
            false,
            DataReaderQos::default(),
        );
        RtpsStatelessReader::new(reader);
    }
}
