use dds_transport::messages::submessages::{
    AckNackSubmessage, DataSubmessage, GapSubmessage, HeartbeatSubmessage,
};

use crate::{
    dcps_psm::InstanceHandle,
    implementation::rtps::utils::clock::{Timer, TimerConstructor},
    infrastructure::qos_policy::ReliabilityQosPolicyKind,
};

use super::{
    history_cache::{RtpsCacheChange, RtpsParameter},
    reader_proxy::{
        BestEffortStatefulWriterSendSubmessage, ChangeForReaderStatusKind,
        ReliableStatefulWriterSendSubmessage, RtpsChangeForReader, RtpsReaderProxy,
    },
    types::{ChangeKind, Count, Guid, GuidPrefix, SequenceNumber},
    writer::RtpsWriter,
};

pub struct RtpsStatefulWriter<T> {
    writer: RtpsWriter,
    matched_readers: Vec<RtpsReaderProxy>,
    heartbeat_timer: T,
    heartbeat_count: Count,
}

impl<T> RtpsStatefulWriter<T> {
    pub fn writer(&self) -> &RtpsWriter {
        &self.writer
    }

    pub fn writer_mut(&mut self) -> &mut RtpsWriter {
        &mut self.writer
    }
}

impl<T> RtpsStatefulWriter<T> {
    pub fn matched_readers(&mut self) -> &mut Vec<RtpsReaderProxy> {
        &mut self.matched_readers
    }
}

impl<T: TimerConstructor> RtpsStatefulWriter<T> {
    pub fn new(writer: RtpsWriter) -> Self {
        Self {
            writer,
            matched_readers: Vec::new(),
            heartbeat_timer: T::new(),
            heartbeat_count: Count(0),
        }
    }
}

impl<T> RtpsStatefulWriter<T> {
    pub fn matched_reader_add(&mut self, mut a_reader_proxy: RtpsReaderProxy) {
        let status = if self.writer.push_mode() {
            ChangeForReaderStatusKind::Unsent
        } else {
            ChangeForReaderStatusKind::Unacknowledged
        };
        for change in self.writer.writer_cache().changes() {
            a_reader_proxy
                .changes_for_reader_mut()
                .push(RtpsChangeForReader::new(
                    status,
                    true,
                    change.sequence_number(),
                ));
        }

        self.matched_readers.push(a_reader_proxy)
    }

    pub fn matched_reader_remove<F>(&mut self, mut f: F)
    where
        F: FnMut(&RtpsReaderProxy) -> bool,
    {
        self.matched_readers.retain(|x| !f(x));
    }

    pub fn matched_reader_lookup(&self, a_reader_guid: Guid) -> Option<&RtpsReaderProxy> {
        self.matched_readers
            .iter()
            .find(|&x| x.remote_reader_guid() == a_reader_guid)
    }

    pub fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}

impl<T> RtpsStatefulWriter<T> {
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

impl<T> RtpsStatefulWriter<T> {
    pub fn add_change(&mut self, change: RtpsCacheChange) {
        let sequence_number = change.sequence_number();
        self.writer.writer_cache().add_change(change);

        for reader_proxy in &mut self.matched_readers {
            let status = if self.writer.push_mode() {
                ChangeForReaderStatusKind::Unsent
            } else {
                ChangeForReaderStatusKind::Unacknowledged
            };
            reader_proxy
                .changes_for_reader_mut()
                .push(RtpsChangeForReader::new(status, true, sequence_number))
        }
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

impl<'a, T> RtpsStatefulWriter<T>
where
    T: Timer,
{
    pub fn send_submessages(
        &'a mut self,
        mut send_data: impl FnMut(&RtpsReaderProxy, DataSubmessage<'a>),
        mut send_gap: impl FnMut(&RtpsReaderProxy, GapSubmessage),
        mut send_heartbeat: impl FnMut(&RtpsReaderProxy, HeartbeatSubmessage),
    ) {
        let time_for_heartbeat = self.heartbeat_timer.elapsed()
            >= std::time::Duration::from_secs(self.writer.heartbeat_period().sec() as u64)
                + std::time::Duration::from_nanos(self.writer.heartbeat_period().nanosec() as u64);
        if time_for_heartbeat {
            self.heartbeat_timer.reset();
            self.heartbeat_count = Count(self.heartbeat_count.0.wrapping_add(1));
        }

        let reliability_level = &self.writer.get_qos().reliability.kind;
        let writer_id = self.writer.guid().entity_id;
        let heartbeat_count = self.heartbeat_count;

        for reader_proxy in self.matched_readers.iter_mut() {
            match reliability_level {
                ReliabilityQosPolicyKind::BestEffortReliabilityQos => {
                    while let Some(send_submessage) =
                        reader_proxy.best_effort_send_unsent_changes(&self.writer.writer_cache)
                    {
                        match send_submessage {
                            BestEffortStatefulWriterSendSubmessage::Data(data) => {
                                send_data(reader_proxy, data)
                            }
                            BestEffortStatefulWriterSendSubmessage::Gap(gap) => {
                                send_gap(reader_proxy, gap)
                            }
                        }
                    }
                }
                ReliabilityQosPolicyKind::ReliableReliabilityQos => {
                    if time_for_heartbeat {
                        let mut heartbeat =
                            reader_proxy.send_heartbeat(writer_id, &self.writer.writer_cache);
                        heartbeat.count.value = heartbeat_count.0;
                        send_heartbeat(reader_proxy, heartbeat)
                    }

                    while let Some(send_submessage) =
                        reader_proxy.reliable_send_unsent_changes(&self.writer.writer_cache)
                    {
                        match send_submessage {
                            ReliableStatefulWriterSendSubmessage::Data(data) => {
                                send_data(reader_proxy, data)
                            }
                            ReliableStatefulWriterSendSubmessage::Gap(gap) => {
                                send_gap(reader_proxy, gap)
                            }
                        }
                    }
                    while let Some(send_requested_submessage) =
                        reader_proxy.send_requested_changes(&self.writer.writer_cache)
                    {
                        match send_requested_submessage {
                            ReliableStatefulWriterSendSubmessage::Data(data) => {
                                send_data(reader_proxy, data)
                            }
                            ReliableStatefulWriterSendSubmessage::Gap(gap) => {
                                send_gap(reader_proxy, gap)
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<T> RtpsStatefulWriter<T> {
    pub fn on_acknack_submessage_received(
        &mut self,
        acknack_submessage: &AckNackSubmessage,
        source_guid_prefix: GuidPrefix,
    ) {
        if self.writer.get_qos().reliability.kind
            == ReliabilityQosPolicyKind::ReliableReliabilityQos
        {
            let reader_guid = Guid::new(
                source_guid_prefix,
                acknack_submessage.reader_id.value.into(),
            );

            if let Some(reader_proxy) = self
                .matched_readers
                .iter_mut()
                .find(|x| x.remote_reader_guid() == reader_guid)
            {
                reader_proxy.reliable_receive_acknack(acknack_submessage);
            }
        }
    }
}
