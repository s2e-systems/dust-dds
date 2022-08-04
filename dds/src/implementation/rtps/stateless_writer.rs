use dds_transport::{
    messages::submessages::{
        AckNackSubmessage, DataSubmessage, GapSubmessage, HeartbeatSubmessage,
    },
    types::Locator,
};

use crate::{
    dcps_psm::InstanceHandle,
    implementation::rtps::utils::clock::{Timer, TimerConstructor},
};

use super::{
    history_cache::{RtpsCacheChange, RtpsParameter},
    reader_locator::{BestEffortStatelessWriterSendSubmessage, RtpsReaderLocator},
    types::{ChangeKind, Count, EntityId, ReliabilityKind, SequenceNumber, ENTITYID_UNKNOWN},
    writer::RtpsWriter,
};

pub struct RtpsStatelessWriter<T> {
    writer: RtpsWriter,
    reader_locators: Vec<RtpsReaderLocator>,
    heartbeat_timer: T,
    _heartbeat_count: Count,
}

impl<T> RtpsStatelessWriter<T> {
    pub fn writer(&self) -> &RtpsWriter {
        &self.writer
    }

    pub fn writer_mut(&mut self) -> &mut RtpsWriter {
        &mut self.writer
    }
}

impl<T> RtpsStatelessWriter<T> {
    pub fn reader_locators(&'_ mut self) -> &mut Vec<RtpsReaderLocator> {
        &mut self.reader_locators
    }
}

impl<T: TimerConstructor> RtpsStatelessWriter<T> {
    pub fn new(writer: RtpsWriter) -> Self {
        Self {
            writer,
            reader_locators: Vec::new(),
            heartbeat_timer: T::new(),
            _heartbeat_count: Count(0),
        }
    }
}

impl<T> RtpsStatelessWriter<T> {
    pub fn reader_locator_add(&mut self, mut a_locator: RtpsReaderLocator) {
        *a_locator.unsent_changes_mut() = self
            .writer
            .writer_cache()
            .changes()
            .iter()
            .map(|c| c.sequence_number())
            .collect();
        self.reader_locators.push(a_locator);
    }

    pub fn reader_locator_remove<F>(&mut self, mut f: F)
    where
        F: FnMut(&RtpsReaderLocator) -> bool,
    {
        self.reader_locators.retain(|x| !f(x))
    }

    pub fn reader_locator_lookup(&mut self, locator: &Locator) -> Option<&mut RtpsReaderLocator> {
        self.reader_locators
            .iter_mut()
            .find(|x| x.locator() == *locator)
    }

    pub fn unsent_changes_reset(&mut self) {
        for reader_locator in &mut self.reader_locators {
            reader_locator.unsent_changes_reset()
        }
    }
}

impl<T> RtpsStatelessWriter<T> {
    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: Vec<u8>,
        inline_qos: Vec<RtpsParameter>,
        handle: InstanceHandle,
    ) -> RtpsCacheChange {
        self.writer.new_change(kind, data, inline_qos, handle)
    }
}

impl<T> RtpsStatelessWriter<T> {
    pub fn add_change(&mut self, change: RtpsCacheChange) {
        for reader_locator in &mut self.reader_locators {
            reader_locator
                .unsent_changes_mut()
                .push(change.sequence_number());
        }
        self.writer.writer_cache().add_change(change);
    }

    pub fn remove_change<F>(&mut self, f: F)
    where
        F: FnMut(&RtpsCacheChange) -> bool,
    {
        self.writer.writer_cache().remove_change(f)
    }

    pub fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.writer.writer_cache.get_seq_num_min()
    }

    pub fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.writer.writer_cache.get_seq_num_max()
    }
}

impl<'a, T> RtpsStatelessWriter<T>
where
    T: Timer,
{
    pub fn send_submessages(
        &'a mut self,
        mut send_data: impl FnMut(&RtpsReaderLocator, DataSubmessage<'a>),
        mut send_gap: impl FnMut(&RtpsReaderLocator, GapSubmessage),
        _send_heartbeat: impl FnMut(&RtpsReaderLocator, HeartbeatSubmessage),
    ) {
        let time_for_heartbeat = self.heartbeat_timer.elapsed()
            >= std::time::Duration::from_secs(self.writer.heartbeat_period().sec() as u64)
                + std::time::Duration::from_nanos(self.writer.heartbeat_period().nanosec() as u64);
        if time_for_heartbeat {
            self.heartbeat_timer.reset();
        }
        let reliability_level = self.writer.reliability_level();
        for reader_locator in &mut self.reader_locators {
            match reliability_level {
                ReliabilityKind::BestEffort => {
                    while let Some(send_submessage) =
                        reader_locator.send_unsent_changes(&self.writer.writer_cache)
                    {
                        match send_submessage {
                            BestEffortStatelessWriterSendSubmessage::Data(data) => {
                                send_data(reader_locator, data)
                            }
                            BestEffortStatelessWriterSendSubmessage::Gap(gap) => {
                                send_gap(reader_locator, gap)
                            }
                        }
                    }
                }
                ReliabilityKind::Reliable => todo!(),
            }
        }
    }
}

impl<T> RtpsStatelessWriter<T> {
    pub fn on_acknack_submessage_received(&mut self, acknack_submessage: &AckNackSubmessage) {
        let acknack_reader_id: EntityId = acknack_submessage.reader_id.value.into();
        let message_is_for_me = acknack_reader_id == ENTITYID_UNKNOWN
            || acknack_reader_id == self.writer.guid().entity_id();

        if self.writer.reliability_level() == ReliabilityKind::Reliable && message_is_for_me {
            for reader_locator in self.reader_locators.iter_mut() {
                reader_locator.receive_acknack(acknack_submessage);
            }
        }
    }
}
