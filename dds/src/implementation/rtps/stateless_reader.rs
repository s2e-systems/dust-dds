use crate::infrastructure::qos_policy::ReliabilityQosPolicyKind;

use super::reader::RtpsReader;

pub struct RtpsStatelessReader(RtpsReader);

impl RtpsStatelessReader {
    pub fn new(reader: RtpsReader) -> Self {
        if reader.get_qos().reliability.kind == ReliabilityQosPolicyKind::ReliableReliabilityQos {
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
        dcps_psm::{Duration, DURATION_ZERO},
        implementation::rtps::{
            endpoint::RtpsEndpoint,
            types::{EntityId, Guid, GuidPrefix, TopicKind, USER_DEFINED_READER_NO_KEY},
        },
        infrastructure::{qos::DataReaderQos, qos_policy::ReliabilityQosPolicy},
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
            &[],
            &[],
        );
        let reader = RtpsReader::new(
            endpoint,
            Duration::new(0, 0),
            Duration::new(0, 0),
            false,
            DataReaderQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::ReliableReliabilityQos,
                    max_blocking_time: DURATION_ZERO,
                },
                ..Default::default()
            },
        );
        RtpsStatelessReader::new(reader);
    }
}
