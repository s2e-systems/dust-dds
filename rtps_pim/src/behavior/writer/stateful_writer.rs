use core::{
    iter::FromIterator,
    ops::{Deref, DerefMut},
};

use crate::{
    behavior::types::Duration,
    messages::{
        submessage_elements::{
            CountSubmessageElement, EntityIdSubmessageElement, ParameterListSubmessageElement,
            SequenceNumberSetSubmessageElement, SequenceNumberSubmessageElement,
            SerializedDataSubmessageElement,
        },
        submessages::{AckNackSubmessage, DataSubmessage, GapSubmessage, HeartbeatSubmessage},
        types::Count,
    },
    structure::{
        types::{ChangeKind, Guid, ReliabilityKind, SequenceNumber, TopicKind, ENTITYID_UNKNOWN},
        RtpsHistoryCacheOperations,
    },
};

use super::{
    reader_proxy::{RtpsReaderProxy, RtpsReaderProxyOperations},
    writer::RtpsWriter,
};

pub struct RtpsStatefulWriter<L, C, R> {
    writer: RtpsWriter<L, C>,
    pub matched_readers: R,
}

impl<L, C, R> Deref for RtpsStatefulWriter<L, C, R> {
    type Target = RtpsWriter<L, C>;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl<L, C, R> DerefMut for RtpsStatefulWriter<L, C, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl<L, C, R> RtpsStatefulWriter<L, C, R>
where
    R: Default,
    C: for<'a> RtpsHistoryCacheOperations<'a>,
{
    pub fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: L,
        multicast_locator_list: L,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_size_serialized: Option<i32>,
    ) -> Self {
        Self {
            writer: RtpsWriter::new(
                guid,
                topic_kind,
                reliability_level,
                unicast_locator_list,
                multicast_locator_list,
                push_mode,
                heartbeat_period,
                nack_response_delay,
                nack_suppression_duration,
                data_max_size_serialized,
            ),
            matched_readers: R::default(),
        }
    }
}

pub trait RtpsStatefulWriterOperations<L> {
    fn matched_reader_add(&mut self, a_reader_proxy: RtpsReaderProxy<L>);

    fn matched_reader_remove(&mut self, reader_proxy_guid: &Guid);

    fn matched_reader_lookup(&self, a_reader_guid: &Guid) -> Option<&RtpsReaderProxy<L>>;

    fn is_acked_by_all(&self) -> bool;
}

impl<L, C, R, RP, P, D, SV> RtpsStatefulWriter<L, C, R>
where
    for<'b> &'b mut R: IntoIterator<Item = &'b mut RP>,
    RP: RtpsReaderProxyOperations<SequenceNumberVector = SV> + Deref<Target = RtpsReaderProxy<L>>,
    SV: ExactSizeIterator,
    C: for<'a> RtpsHistoryCacheOperations<'a, GetChangeDataType = D, GetChangeParameterType = P>,
{
    pub fn send_unsent_data<S>(
        &mut self,
        send_data: &mut impl FnMut(&RP, DataSubmessage<P, D>),
        send_gap: &mut impl FnMut(&RP, GapSubmessage<S>),
    ) where
        S: FromIterator<SequenceNumber>,
    {
        for reader_proxy in &mut self.matched_readers {
            while let Some(seq_num) = reader_proxy.next_unsent_change() {
                if let Some(change) = self.writer.writer_cache.get_change(&seq_num) {
                    let endianness_flag = true;
                    let inline_qos_flag = true;
                    let (data_flag, key_flag) = match change.kind {
                        ChangeKind::Alive => (true, false),
                        ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => {
                            (false, true)
                        }
                        _ => todo!(),
                    };
                    let non_standard_payload_flag = false;
                    let reader_id = EntityIdSubmessageElement {
                        value: ENTITYID_UNKNOWN,
                    };
                    let writer_id = EntityIdSubmessageElement {
                        value: change.writer_guid.entity_id,
                    };
                    let writer_sn = SequenceNumberSubmessageElement {
                        value: change.sequence_number,
                    };
                    let inline_qos = ParameterListSubmessageElement {
                        parameter: change.inline_qos,
                    };
                    let serialized_payload = SerializedDataSubmessageElement {
                        value: change.data_value,
                    };
                    let data_submessage = DataSubmessage {
                        endianness_flag,
                        inline_qos_flag,
                        data_flag,
                        key_flag,
                        non_standard_payload_flag,
                        reader_id,
                        writer_id,
                        writer_sn,
                        inline_qos,
                        serialized_payload,
                    };
                    send_data(reader_proxy, data_submessage)
                } else {
                    let endianness_flag = true;
                    let reader_id = EntityIdSubmessageElement {
                        value: ENTITYID_UNKNOWN,
                    };
                    let writer_id = EntityIdSubmessageElement {
                        value: self.writer.guid.entity_id,
                    };
                    let gap_start = SequenceNumberSubmessageElement { value: seq_num };
                    let set = core::iter::empty().collect();
                    let gap_list = SequenceNumberSetSubmessageElement { base: seq_num, set };
                    let gap_submessage = GapSubmessage {
                        endianness_flag,
                        reader_id,
                        writer_id,
                        gap_start,
                        gap_list,
                    };
                    send_gap(reader_proxy, gap_submessage)
                }
            }
        }
    }

    pub fn send_heartbeat(
        &mut self,
        count: Count,
        send_heartbeat: &mut impl FnMut(&RP, HeartbeatSubmessage),
    ) {
        for reader_proxy in &mut self.matched_readers {
            let endianness_flag = true;
            let final_flag = false;
            let liveliness_flag = false;
            let reader_id = EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            };
            let writer_id = EntityIdSubmessageElement {
                value: self.writer.guid.entity_id,
            };
            let first_sn = SequenceNumberSubmessageElement {
                value: self.writer.writer_cache.get_seq_num_min().unwrap_or(0),
            };
            let last_sn = SequenceNumberSubmessageElement {
                value: self.writer.writer_cache.get_seq_num_min().unwrap_or(0),
            };
            let count = CountSubmessageElement { value: count };
            let heartbeat_submessage = HeartbeatSubmessage {
                endianness_flag,
                final_flag,
                liveliness_flag,
                reader_id,
                writer_id,
                first_sn,
                last_sn,
                count,
            };
            send_heartbeat(reader_proxy, heartbeat_submessage)
        }
    }

    pub fn process_acknack_submessage(&mut self, acknack: AckNackSubmessage<SV>) {
        for reader_proxy in &mut self.matched_readers {
            if reader_proxy.remote_reader_guid.entity_id == acknack.reader_id.value {
                reader_proxy.acked_changes_set(acknack.reader_sn_state.base - 1);
                reader_proxy.requested_changes_set(acknack.reader_sn_state.set);
                break;
            }
        }
    }

    pub fn send_requested_data<S>(
        &mut self,
        send_data: &mut impl FnMut(&RP, DataSubmessage<P, D>),
        send_gap: &mut impl FnMut(&RP, GapSubmessage<S>),
    ) where
        S: FromIterator<SequenceNumber>,
    {
        for reader_proxy in &mut self.matched_readers {
            // Pushing state
            while let Some(seq_num) = reader_proxy.next_requested_change() {
                if let Some(change) = self.writer.writer_cache.get_change(&seq_num) {
                    let endianness_flag = true;
                    let inline_qos_flag = true;
                    let (data_flag, key_flag) = match change.kind {
                        ChangeKind::Alive => (true, false),
                        ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => {
                            (false, true)
                        }
                        _ => todo!(),
                    };
                    let non_standard_payload_flag = false;
                    let reader_id = EntityIdSubmessageElement {
                        value: ENTITYID_UNKNOWN,
                    };
                    let writer_id = EntityIdSubmessageElement {
                        value: *change.writer_guid.entity_id(),
                    };
                    let writer_sn = SequenceNumberSubmessageElement {
                        value: change.sequence_number,
                    };
                    let inline_qos = ParameterListSubmessageElement {
                        parameter: change.inline_qos,
                    };
                    let serialized_payload = SerializedDataSubmessageElement {
                        value: change.data_value,
                    };
                    let data_submessage = DataSubmessage {
                        endianness_flag,
                        inline_qos_flag,
                        data_flag,
                        key_flag,
                        non_standard_payload_flag,
                        reader_id,
                        writer_id,
                        writer_sn,
                        inline_qos,
                        serialized_payload,
                    };
                    send_data(reader_proxy, data_submessage)
                } else {
                    let endianness_flag = true;
                    let reader_id = EntityIdSubmessageElement {
                        value: ENTITYID_UNKNOWN,
                    };
                    let writer_id = EntityIdSubmessageElement {
                        value: ENTITYID_UNKNOWN,
                    };
                    let gap_start = SequenceNumberSubmessageElement { value: seq_num };
                    let set = core::iter::empty().collect();
                    let gap_list = SequenceNumberSetSubmessageElement { base: seq_num, set };
                    let gap_submessage = GapSubmessage {
                        endianness_flag,
                        reader_id,
                        writer_id,
                        gap_start,
                        gap_list,
                    };
                    send_gap(reader_proxy, gap_submessage)
                }
            }
        }
    }
}
