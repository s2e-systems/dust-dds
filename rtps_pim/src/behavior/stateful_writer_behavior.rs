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

use super::{writer::{
    change_for_reader::RtpsChangeForReaderAttributes, reader_proxy::{RtpsReaderProxyOperations, RtpsReaderProxyAttributes},
}, reader::reader::RtpsReaderAttributes};

pub struct BestEffortStatefulWriterBehavior;

impl BestEffortStatefulWriterBehavior {
    /// 8.4.9.1.4 Transition T4
    pub fn send_unsent_changes<C, P, D, S>(
        reader_proxy: &mut (impl RtpsReaderProxyOperations<ChangeForReaderType = C> + RtpsReaderProxyAttributes),
        mut send_data: impl FnMut(DataSubmessage<P, D>),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) where
        C: RtpsCacheChangeAttributes + RtpsChangeForReaderAttributes,
        C: Into<DataSubmessage<P, D>>,
        S: FromIterator<SequenceNumber>,
    {
        let reader_id = reader_proxy.remote_reader_guid().entity_id();
        while let Some(change_for_reader) = reader_proxy.next_unsent_change() {
            if change_for_reader.is_relevant() {
                let mut data_submessage: DataSubmessage<P, D> = change_for_reader.into();
                data_submessage.reader_id = EntityIdSubmessageElement { value: reader_id };
                send_data(data_submessage)
            } else {
                let seq_num = change_for_reader.sequence_number();
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
    pub fn send_unsent_changes<C, P, D, S>(
        reader_proxy: &mut (impl RtpsReaderProxyOperations<ChangeForReaderType = C> + RtpsReaderProxyAttributes),
        mut send_data: impl FnMut(DataSubmessage<P, D>),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) where
        C: RtpsCacheChangeAttributes + RtpsChangeForReaderAttributes,
        C: Into<DataSubmessage<P, D>>,
        S: FromIterator<SequenceNumber>,
    {
        let reader_id = reader_proxy.remote_reader_guid().entity_id();
        while let Some(change_for_reader) = reader_proxy.next_unsent_change() {
            if change_for_reader.is_relevant() {
                let mut data_submessage = change_for_reader.into();
                data_submessage.reader_id.value = reader_id;
                send_data(data_submessage)
            } else {
                let seq_num = change_for_reader.sequence_number();
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
            value: writer_cache.get_seq_num_min().unwrap_or(1),
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

    /// 8.4.9.2.10 Transition T12
    pub fn send_requested_changes<'a, C, P, D, S>(
        reader_proxy: &mut (impl RtpsReaderProxyOperations<ChangeForReaderType = C> + RtpsReaderProxyAttributes),
        mut send_data: impl FnMut(DataSubmessage<P, D>),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) where
        C: Into<DataSubmessage<P, D>> + RtpsChangeForReaderAttributes + RtpsCacheChangeAttributes,
        S: FromIterator<SequenceNumber>,
    {
        let reader_id = reader_proxy.remote_reader_guid().entity_id();
        while let Some(change_for_reader) = reader_proxy.next_requested_change() {
            if change_for_reader.is_relevant() {
                let mut data_submessage = change_for_reader.into();
                data_submessage.reader_id.value = reader_id;
                send_data(data_submessage)
            } else {
                let seq_num = change_for_reader.sequence_number();
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
