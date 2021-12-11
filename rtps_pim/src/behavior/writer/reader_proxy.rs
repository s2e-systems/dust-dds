use core::iter::FromIterator;

use crate::{
    messages::{
        submessage_elements::{
            CountSubmessageElement, EntityIdSubmessageElement, ParameterListSubmessageElement,
            SequenceNumberSetSubmessageElement, SequenceNumberSubmessageElement,
            SerializedDataSubmessageElement,
        },
        submessages::{DataSubmessage, GapSubmessage, HeartbeatSubmessage},
        types::Count,
    },
    structure::{
        history_cache::{RtpsHistoryCacheGetChange, RtpsHistoryCacheOperations},
        types::{ChangeKind, EntityId, Guid, SequenceNumber, ENTITYID_UNKNOWN},
    },
};

use super::{stateful_writer::StatefulWriterBehaviorPerProxy, writer::RtpsWriter};

#[derive(Debug, PartialEq)]
pub struct RtpsReaderProxy<L> {
    pub remote_reader_guid: Guid,
    pub remote_group_entity_id: EntityId,
    pub unicast_locator_list: L,
    pub multicast_locator_list: L,
    pub expects_inline_qos: bool,
}

impl<L> RtpsReaderProxy<L> {
    pub fn new(
        remote_reader_guid: Guid,
        remote_group_entity_id: EntityId,
        unicast_locator_list: L,
        multicast_locator_list: L,
        expects_inline_qos: bool,
    ) -> Self {
        Self {
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
        }
    }
}

pub trait RtpsReaderProxyTrait {
    fn guid(&self) -> &Guid;
}

pub trait RtpsReaderProxyOperations {
    type SequenceNumberVector;

    fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber);
    fn next_requested_change(&mut self) -> Option<SequenceNumber>;
    fn next_unsent_change(
        &mut self,
        last_change_sequence_number: &SequenceNumber,
    ) -> Option<SequenceNumber>;
    fn unsent_changes(
        &self,
        last_change_sequence_number: &SequenceNumber,
    ) -> Self::SequenceNumberVector;
    fn requested_changes(&self) -> Self::SequenceNumberVector;
    fn requested_changes_set(
        &mut self,
        req_seq_num_set: &[SequenceNumber],
        last_change_sequence_number: &SequenceNumber,
    );
    fn unacked_changes(
        &self,
        last_change_sequence_number: &SequenceNumber,
    ) -> Self::SequenceNumberVector;
}

impl<'a, S> dyn RtpsReaderProxyOperations<SequenceNumberVector = S> + 'a
where
    S: FromIterator<SequenceNumber>,
{
    pub fn send_unsent_data<P, D, L, C>(
        &mut self,
        writer: &'a RtpsWriter<L, C>,
        send_data: &mut dyn FnMut(DataSubmessage<P, D>),
        send_gap: &mut dyn FnMut(GapSubmessage<S>),
    ) where
        C: RtpsHistoryCacheGetChange<'a, P, D>,
    {
        while let Some(seq_num) = self.next_unsent_change(&writer.last_change_sequence_number) {
            if let Some(change) = writer.writer_cache.get_change(&seq_num) {
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
                send_data(data_submessage)
            } else {
                let endianness_flag = true;
                let reader_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let writer_id = EntityIdSubmessageElement {
                    value: writer.endpoint.entity.guid.entity_id,
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
                send_gap(gap_submessage)
            }
        }
    }
}

impl<'a, S, P, D, L, C, T> StatefulWriterBehaviorPerProxy<'a, S, P, D, L, C> for T
where
    T: RtpsReaderProxyOperations<SequenceNumberVector = S>,
    S: FromIterator<SequenceNumber> + AsRef<[SequenceNumber]>,
    C: RtpsHistoryCacheGetChange<'a, P, D> + RtpsHistoryCacheOperations,
{
    fn send_unsent_data(
        &mut self,
        writer: &'a RtpsWriter<L, C>,
        send_data: &mut dyn FnMut(DataSubmessage<P, D>),
        send_gap: &mut dyn FnMut(GapSubmessage<S>),
    ) {
        let last_change_sequence_number = writer.last_change_sequence_number;

        while let Some(seq_num) = self.next_unsent_change(&last_change_sequence_number) {
            if let Some(change) = writer.writer_cache.get_change(&seq_num) {
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
                send_data(data_submessage)
            } else {
                let endianness_flag = true;
                let reader_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let writer_id = EntityIdSubmessageElement {
                    value: writer.endpoint.entity.guid.entity_id,
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
                send_gap(gap_submessage)
            }
        }
    }

    fn send_requested_data(
        &mut self,
        writer: &'a RtpsWriter<L, C>,
        send_data: &mut dyn FnMut(DataSubmessage<P, D>),
        send_gap: &mut dyn FnMut(GapSubmessage<S>),
    ) {
        // Pushing state
        while let Some(seq_num) = self.next_requested_change() {
            if let Some(change) = writer.writer_cache.get_change(&seq_num) {
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
                send_data(data_submessage)
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
                send_gap(gap_submessage)
            }
        }
    }

    fn send_heartbeat(
        &self,
        writer: &RtpsWriter<L, C>,
        heartbeat_count: Count,
        send_heartbeat: &mut dyn FnMut(HeartbeatSubmessage),
    ) {
        let endianness_flag = true;
        let final_flag = false;
        let liveliness_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: ENTITYID_UNKNOWN,
        };
        let writer_id = EntityIdSubmessageElement {
            value: writer.endpoint.entity.guid.entity_id,
        };
        let first_sn = SequenceNumberSubmessageElement {
            value: writer.writer_cache.get_seq_num_min().unwrap_or(0),
        };
        let last_sn = SequenceNumberSubmessageElement {
            value: writer.writer_cache.get_seq_num_min().unwrap_or(0),
        };
        let count = CountSubmessageElement {
            value: heartbeat_count,
        };
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
        send_heartbeat(heartbeat_submessage)
    }

    fn process_acknack_submessage(
        &mut self,
        writer: &'a RtpsWriter<L, C>,
        acknack: &crate::messages::submessages::AckNackSubmessage<S>,
    ) {
        // TODO: maybe do this outside:
        // if self.remote_reader_guid.entity_id == acknack.reader_id.value {
        self.acked_changes_set(acknack.reader_sn_state.base - 1);
        self.requested_changes_set(
            acknack.reader_sn_state.set.as_ref(),
            &writer.last_change_sequence_number,
        );
        // }
    }
}
