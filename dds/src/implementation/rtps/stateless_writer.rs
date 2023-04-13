use crate::{
    infrastructure::{
        error::DdsResult,
        instance::{InstanceHandle, HANDLE_NIL},
        qos_policy::ReliabilityQosPolicyKind,
        time::Time,
    },
    topic_definition::type_support::DdsSerializedKey,
};

use super::{
    history_cache::RtpsWriterCacheChange,
    messages::overall_structure::RtpsMessageHeader,
    reader_locator::RtpsReaderLocator,
    transport::TransportWrite,
    types::{ChangeKind, Count},
    writer::RtpsWriter,
};

pub struct RtpsStatelessWriter {
    writer: RtpsWriter,
    reader_locators: Vec<RtpsReaderLocator>,
    _heartbeat_count: Count,
}

impl RtpsStatelessWriter {
    pub fn new(writer: RtpsWriter) -> Self {
        Self {
            writer,
            reader_locators: Vec::new(),
            _heartbeat_count: Count::new(0),
        }
    }

    pub fn reader_locator_add(&mut self, mut a_locator: RtpsReaderLocator) {
        *a_locator.unsent_changes_mut() = self
            .writer
            .writer_cache()
            .change_list()
            .iter()
            .map(|c| c.sequence_number())
            .collect();
        self.reader_locators.push(a_locator);
    }

    fn add_change(&mut self, change: RtpsWriterCacheChange) {
        for reader_locator in &mut self.reader_locators {
            reader_locator
                .unsent_changes_mut()
                .push(change.sequence_number());
        }
        self.writer.writer_cache_mut().add_change(change);
    }

    pub fn write_w_timestamp(
        &mut self,
        serialized_data: Vec<u8>,
        instance_serialized_key: DdsSerializedKey,
        _handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let handle = self
            .writer
            .register_instance_w_timestamp(instance_serialized_key, timestamp)?
            .unwrap_or(HANDLE_NIL);
        let change = self.writer.new_change(
            ChangeKind::Alive,
            serialized_data,
            vec![],
            handle,
            timestamp,
        );

        self.add_change(change);

        Ok(())
    }

    pub fn send_message(&mut self, header: RtpsMessageHeader, transport: &mut impl TransportWrite) {
        match self.writer.get_qos().reliability.kind {
            ReliabilityQosPolicyKind::BestEffort => {
                for rl in self.reader_locators.iter_mut() {
                    rl.send_message(
                        self.writer.writer_cache(),
                        self.writer.guid().entity_id(),
                        header,
                        transport,
                    );
                }
            }
            ReliabilityQosPolicyKind::Reliable => unimplemented!(),
        }
    }
}
