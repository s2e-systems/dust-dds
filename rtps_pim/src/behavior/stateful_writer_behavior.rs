use core::iter::FromIterator;

/// This file implements the behaviors described in 8.4.9 RTPS StatefulWriter Behavior
use crate::{
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
        cache_change::RtpsCacheChangeAttributes,
        history_cache::{RtpsHistoryCacheAttributes, RtpsHistoryCacheOperations},
        types::{ChangeKind, EntityId, SequenceNumber, ENTITYID_UNKNOWN},
    },
};

use super::writer::reader_proxy::RtpsReaderProxyOperations;

pub struct BestEffortStatefulWriterBehavior;

impl BestEffortStatefulWriterBehavior {
    /// 8.4.9.1.4 Transition T4
    pub fn send_unsent_changes<'a, CacheChange, S, P, D, ChangeForReader>(
        reader_proxy: &mut impl RtpsReaderProxyOperations<ChangeForReaderType = ChangeForReader>,
        writer_cache: &'a impl RtpsHistoryCacheAttributes<CacheChangeType = CacheChange>,
        reader_id: EntityId,
        mut send_data: impl FnMut(DataSubmessage<P, D>),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) where
        CacheChange: RtpsCacheChangeAttributes<'a> + 'a,
        for<'b> &'b ChangeForReader: Into<SequenceNumber>,
        &'a <CacheChange as RtpsCacheChangeAttributes<'a>>::DataType: Into<D>,
        &'a <CacheChange as RtpsCacheChangeAttributes<'a>>::ParameterListType: Into<P>,
        S: FromIterator<SequenceNumber>,
    {
        while let Some(change_for_reader) = reader_proxy.next_unsent_change() {
            let seq_num = (&*change_for_reader).into();
            let change = writer_cache
                .changes()
                .iter()
                .filter(|cc| cc.sequence_number() == seq_num)
                .next();
            if let Some(change) = change {
                let endianness_flag = true;
                let inline_qos_flag = true;
                let (data_flag, key_flag) = match change.kind() {
                    ChangeKind::Alive => (true, false),
                    ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => {
                        (false, true)
                    }
                    _ => todo!(),
                };
                let non_standard_payload_flag = false;
                let reader_id = EntityIdSubmessageElement { value: reader_id };
                let writer_id = EntityIdSubmessageElement {
                    value: change.writer_guid().entity_id(),
                };
                let writer_sn = SequenceNumberSubmessageElement {
                    value: change.sequence_number(),
                };
                let inline_qos = ParameterListSubmessageElement {
                    parameter: change.inline_qos().into(),
                };
                let serialized_payload = SerializedDataSubmessageElement {
                    value: change.data_value().into(),
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
                let reader_id = EntityIdSubmessageElement { value: reader_id };
                let writer_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let gap_start = SequenceNumberSubmessageElement { value: seq_num };
                let gap_list = SequenceNumberSetSubmessageElement {
                    base: seq_num,
                    set: core::iter::empty().collect(),
                };
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

/// This struct is a wrapper for the implementation of the behaviors described in 8.4.9.2 Reliable StatefulWriter Behavior
pub struct ReliableStatefulWriterBehavior;

impl ReliableStatefulWriterBehavior {
    /// Implement 8.4.9.2.4 Transition T4
    pub fn send_unsent_changes<'a, CacheChange, S, P, D, ChangeForReader>(
        reader_proxy: &mut impl RtpsReaderProxyOperations<ChangeForReaderType = ChangeForReader>,
        writer_cache: &'a impl RtpsHistoryCacheAttributes<CacheChangeType = CacheChange>,
        reader_id: EntityId,
        mut send_data: impl FnMut(DataSubmessage<P, D>),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) where
        CacheChange: RtpsCacheChangeAttributes<'a> + 'a,
        for<'b> &'b ChangeForReader: Into<SequenceNumber>,
        &'a <CacheChange as RtpsCacheChangeAttributes<'a>>::DataType: Into<D>,
        &'a <CacheChange as RtpsCacheChangeAttributes<'a>>::ParameterListType: Into<P>,
        S: FromIterator<SequenceNumber>,
    {
        while let Some(change_for_reader) = reader_proxy.next_unsent_change() {
            let seq_num = (&*change_for_reader).into();
            let change = writer_cache
                .changes()
                .iter()
                .filter(|cc| cc.sequence_number() == seq_num)
                .next();
            if let Some(change) = change {
                let endianness_flag = true;
                let inline_qos_flag = true;
                let (data_flag, key_flag) = match change.kind() {
                    ChangeKind::Alive => (true, false),
                    ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => {
                        (false, true)
                    }
                    _ => todo!(),
                };
                let non_standard_payload_flag = false;
                let reader_id = EntityIdSubmessageElement { value: reader_id };
                let writer_id = EntityIdSubmessageElement {
                    value: change.writer_guid().entity_id(),
                };
                let writer_sn = SequenceNumberSubmessageElement {
                    value: change.sequence_number(),
                };
                let inline_qos = ParameterListSubmessageElement {
                    parameter: change.inline_qos().into(),
                };
                let serialized_payload = SerializedDataSubmessageElement {
                    value: change.data_value().into(),
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
                let reader_id = EntityIdSubmessageElement { value: reader_id };
                let writer_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let gap_start = SequenceNumberSubmessageElement { value: seq_num };
                let gap_list = SequenceNumberSetSubmessageElement {
                    base: seq_num,
                    set: core::iter::empty().collect(),
                };
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

    /// 8.4.9.2.7 Transition T7
    pub fn send_heartbeat(
        writer_cache: &impl RtpsHistoryCacheOperations,
        writer_id: EntityId,
        heartbeat_count: Count,
        mut send_heartbeat: impl FnMut(HeartbeatSubmessage),
    ) {
        let endianness_flag = true;
        let final_flag = false;
        let liveliness_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: ENTITYID_UNKNOWN,
        };
        let writer_id = EntityIdSubmessageElement { value: writer_id };
        let first_sn = SequenceNumberSubmessageElement {
            value: writer_cache.get_seq_num_min().unwrap_or(0),
        };
        let last_sn = SequenceNumberSubmessageElement {
            value: writer_cache.get_seq_num_max().unwrap_or(0),
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

    /// 8.4.9.2.8 Transition T8
    pub fn receive_acknack<S>(
        reader_proxy: &mut impl RtpsReaderProxyOperations,
        acknack: &AckNackSubmessage<S>,
    ) where
        S: AsRef<[SequenceNumber]>,
    {
        reader_proxy.acked_changes_set(acknack.reader_sn_state.base - 1);
        reader_proxy.requested_changes_set(acknack.reader_sn_state.set.as_ref());
    }

    /// 8.4.8.2.10 Transition T10
    pub fn send_requested_changes<'a, CacheChange, S, P, D>(
        reader_proxy: &mut impl RtpsReaderProxyOperations<ChangeForReaderType = SequenceNumber>,
        writer_cache: &'a impl RtpsHistoryCacheAttributes<CacheChangeType = CacheChange>,
        reader_id: EntityId,
        mut send_data: impl FnMut(DataSubmessage<P, D>),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) where
        CacheChange: RtpsCacheChangeAttributes<'a> + 'a,
        &'a <CacheChange as RtpsCacheChangeAttributes<'a>>::DataType: Into<D>,
        &'a <CacheChange as RtpsCacheChangeAttributes<'a>>::ParameterListType: Into<P>,
        S: FromIterator<SequenceNumber>,
    {
        while let Some(&mut seq_num) = reader_proxy.next_requested_change() {
            let change = writer_cache
                .changes()
                .iter()
                .filter(|cc| cc.sequence_number() == seq_num)
                .next();
            if let Some(change) = change {
                let endianness_flag = true;
                let inline_qos_flag = true;
                let (data_flag, key_flag) = match change.kind() {
                    ChangeKind::Alive => (true, false),
                    ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => {
                        (false, true)
                    }
                    _ => todo!(),
                };
                let non_standard_payload_flag = false;
                let reader_id = EntityIdSubmessageElement { value: reader_id };
                let writer_id = EntityIdSubmessageElement {
                    value: change.writer_guid().entity_id(),
                };
                let writer_sn = SequenceNumberSubmessageElement {
                    value: change.sequence_number(),
                };
                let inline_qos = ParameterListSubmessageElement {
                    parameter: change.inline_qos().into(),
                };
                let serialized_payload = SerializedDataSubmessageElement {
                    value: change.data_value().into(),
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
                let reader_id = EntityIdSubmessageElement { value: reader_id };
                let writer_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let gap_start = SequenceNumberSubmessageElement { value: seq_num };
                let gap_list = SequenceNumberSetSubmessageElement {
                    base: seq_num,
                    set: core::iter::empty().collect(),
                };
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
