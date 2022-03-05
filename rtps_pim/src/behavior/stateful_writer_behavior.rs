use core::iter::FromIterator;

/// This file implements the behaviors described in 8.4.9 RTPS StatefulWriter Behavior
use crate::{
    messages::{
        submessage_elements::{
            CountSubmessageElementConstructor, EntityIdSubmessageElement, Parameter,
            ParameterListSubmessageElement, ParameterListSubmessageElementConstructor,
            SequenceNumberSetSubmessageElement, SequenceNumberSetSubmessageElementAttributes,
            SequenceNumberSetSubmessageElementConstructor, SequenceNumberSubmessageElement,
            SequenceNumberSubmessageElementConstructor, SerializedDataSubmessageElement,
        },
        submessages::{
            AckNackSubmessageAttributes, DataSubmessageConstructor, GapSubmessageConstructor,
            HeartbeatSubmessageConstructor,
        },
        types::Count,
    },
    structure::{
        cache_change::RtpsCacheChangeAttributes,
        history_cache::{RtpsHistoryCacheAttributes, RtpsHistoryCacheOperations},
        types::{ChangeKind, Guid, SequenceNumber, ENTITYID_UNKNOWN},
    },
};

use super::writer::reader_proxy::{RtpsReaderProxyAttributes, RtpsReaderProxyOperations};

pub enum StatefulWriterBehavior<'a, R, C> {
    BestEffort(BestEffortStatefulWriterBehavior<'a, R, C>),
    Reliable(ReliableStatefulWriterBehavior<'a, R, C>),
}

pub struct BestEffortStatefulWriterBehavior<'a, R, C> {
    pub reader_proxy: R,
    pub writer_cache: &'a C,
    pub last_change_sequence_number: SequenceNumber,
}

impl<'a, R, C> BestEffortStatefulWriterBehavior<'a, R, C> {
    /// Implement 8.4.9.1.4 Transition T4
    pub fn send_unsent_changes<
        Data,
        EntityIdElement,
        SequenceNumberElement,
        CacheChange,
        Gap,
        SequenceNumberSetElement,
        ParameterListElement,
        S,
        P,
    >(
        &mut self,
        mut send_data: impl FnMut(Data),
        mut send_gap: impl FnMut(Gap),
    ) where
        R: RtpsReaderProxyOperations<ChangeForReaderType = SequenceNumber>
            + RtpsReaderProxyAttributes,
        Data: DataSubmessageConstructor<'a, P>,
        ParameterListElement: ParameterListSubmessageElementConstructor<'a>,
        C: RtpsHistoryCacheAttributes<CacheChangeType = CacheChange>,
        CacheChange: RtpsCacheChangeAttributes<'a, DataType = [u8]> + 'a,
        &'a <CacheChange as RtpsCacheChangeAttributes<'a>>::ParameterListType:
            IntoIterator<Item = Parameter<'a>> + 'a,
        SequenceNumberElement: SequenceNumberSubmessageElementConstructor,
        SequenceNumberSetElement: SequenceNumberSetSubmessageElementConstructor<'a>,
        Gap: GapSubmessageConstructor<S>,
        S: FromIterator<SequenceNumber>,
        P: FromIterator<Parameter<'a>>,
    {
        while let Some(seq_num) = self.reader_proxy.next_unsent_change() {
            let change = self
                .writer_cache
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
                let reader_id = EntityIdSubmessageElement {
                    value: self.reader_proxy.remote_reader_guid().entity_id(),
                };
                let writer_id = EntityIdSubmessageElement {
                    value: change.writer_guid().entity_id(),
                };
                let writer_sn = SequenceNumberSubmessageElement {
                    value: change.sequence_number(),
                };
                let inline_qos = ParameterListSubmessageElement {
                    parameter: change.inline_qos().into_iter().collect(),
                };
                let serialized_payload = SerializedDataSubmessageElement {
                    value: change.data_value(),
                };
                let data_submessage = Data::new(
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
                );
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
                let gap_list = SequenceNumberSetSubmessageElement {
                    base: seq_num,
                    set: core::iter::empty().collect(),
                };
                let gap_submessage =
                    Gap::new(endianness_flag, reader_id, writer_id, gap_start, gap_list);
                send_gap(gap_submessage)
            }
        }
    }
}

/// This struct is a wrapper for the implementation of the behaviors described in 8.4.9.2 Reliable StatefulWriter Behavior
pub struct ReliableStatefulWriterBehavior<'a, R, C> {
    pub reader_proxy: R,
    pub writer_cache: &'a C,
    pub last_change_sequence_number: SequenceNumber,
    pub writer_guid: Guid,
    pub heartbeat_count: Count,
    pub after_heartbeat_period: bool,
}

impl<'a, R, C> ReliableStatefulWriterBehavior<'a, R, C> {
    /// Implement 8.4.9.2.4 Transition T4
    pub fn send_unsent_changes<Data, CacheChange, Gap, S, P>(
        &mut self,
        mut send_data: impl FnMut(Data),
        mut send_gap: impl FnMut(Gap),
    ) where
        R: RtpsReaderProxyOperations<ChangeForReaderType = SequenceNumber>,
        C: RtpsHistoryCacheAttributes<CacheChangeType = CacheChange>,
        Data: DataSubmessageConstructor<'a, P>,
        CacheChange: RtpsCacheChangeAttributes<'a, DataType = [u8]> + 'a,
        &'a <CacheChange as RtpsCacheChangeAttributes<'a>>::ParameterListType:
            IntoIterator<Item = Parameter<'a>> + 'a,
        Gap: GapSubmessageConstructor<S>,
        S: FromIterator<SequenceNumber>,
        P: FromIterator<Parameter<'a>>,
    {
        while let Some(seq_num) = self.reader_proxy.next_unsent_change() {
            let change = self
                .writer_cache
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
                let reader_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                // EntityIdElement::new(self.reader_proxy.remote_reader_guid().entity_id());
                let writer_id = EntityIdSubmessageElement {
                    value: change.writer_guid().entity_id(),
                };
                let writer_sn = SequenceNumberSubmessageElement {
                    value: change.sequence_number(),
                };
                let inline_qos = ParameterListSubmessageElement {
                    parameter: change.inline_qos().into_iter().collect(),
                };
                let serialized_payload = SerializedDataSubmessageElement {
                    value: change.data_value(),
                };
                let data_submessage = Data::new(
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
                );
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
                let gap_list = SequenceNumberSetSubmessageElement {
                    base: seq_num,
                    set: core::iter::empty().collect(),
                };
                let gap_submessage =
                    Gap::new(endianness_flag, reader_id, writer_id, gap_start, gap_list);
                send_gap(gap_submessage)
            }
        }
    }

    /// Implement 8.4.9.2.7 Transition T7
    pub fn send_heartbeat<Heartbeat, CountElement, SequenceNumberElement>(
        &mut self,
        mut send_heartbeat: impl FnMut(Heartbeat),
    ) where
        C: RtpsHistoryCacheOperations,
        Heartbeat: HeartbeatSubmessageConstructor<
            SequenceNumberSubmessageElementType = SequenceNumberElement,
            CountSubmessageElementType = CountElement,
        >,

        CountElement: CountSubmessageElementConstructor,
        SequenceNumberElement: SequenceNumberSubmessageElementConstructor,
    {
        if self.after_heartbeat_period {
            let endianness_flag = true;
            let final_flag = false;
            let liveliness_flag = false;
            let reader_id = EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            };
            let writer_id = EntityIdSubmessageElement {
                value: self.writer_guid.entity_id,
            };
            let first_sn =
                SequenceNumberElement::new(self.writer_cache.get_seq_num_min().unwrap_or(0));
            let last_sn =
                SequenceNumberElement::new(self.writer_cache.get_seq_num_min().unwrap_or(0));
            let count = CountElement::new(self.heartbeat_count);
            let heartbeat_submessage = Heartbeat::new(
                endianness_flag,
                final_flag,
                liveliness_flag,
                reader_id,
                writer_id,
                first_sn,
                last_sn,
                count,
            );
            send_heartbeat(heartbeat_submessage)
        }
    }

    /// Implement 8.4.9.2.8 Transition T8
    pub fn process_acknack<S>(
        &mut self,
        acknack: &impl AckNackSubmessageAttributes<
            SequenceNumberSetSubmessageElementType = impl SequenceNumberSetSubmessageElementAttributes,
        >,
    ) where
        R: RtpsReaderProxyOperations + RtpsReaderProxyAttributes,
        S: AsRef<[SequenceNumber]>,
    {
        self.reader_proxy
            .acked_changes_set(acknack.reader_sn_state().base() - 1);
        self.reader_proxy
            .requested_changes_set(acknack.reader_sn_state().set());
    }

    /// Implement 8.4.8.2.10 Transition T10
    pub fn send_requested_changes<
        Data,
        EntityIdElement,
        SequenceNumberElement,
        CacheChange,
        Gap,
        SequenceNumberSetElement,
        ParameterListElement,
        S,
        P,
    >(
        &mut self,
        mut send_data: impl FnMut(Data),
        mut send_gap: impl FnMut(Gap),
    ) where
        R: RtpsReaderProxyOperations<ChangeForReaderType = SequenceNumber>,
        C: RtpsHistoryCacheAttributes<CacheChangeType = CacheChange>,
        Data: DataSubmessageConstructor<'a, P>,
        CacheChange: RtpsCacheChangeAttributes<'a, DataType = [u8]> + 'a,
        &'a <CacheChange as RtpsCacheChangeAttributes<'a>>::ParameterListType:
            IntoIterator<Item = Parameter<'a>> + 'a,
        SequenceNumberElement: SequenceNumberSubmessageElementConstructor,
        ParameterListElement: ParameterListSubmessageElementConstructor<'a>,
        SequenceNumberSetElement: SequenceNumberSetSubmessageElementConstructor<'a>,
        Gap: GapSubmessageConstructor<S>,
        S: FromIterator<SequenceNumber>,
        P: FromIterator<Parameter<'a>>,
    {
        while let Some(seq_num) = self.reader_proxy.next_requested_change() {
            let change = self
                .writer_cache
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
                let reader_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let writer_id = EntityIdSubmessageElement {
                    value: change.writer_guid().entity_id(),
                };
                let writer_sn = SequenceNumberSubmessageElement {
                    value: change.sequence_number(),
                };
                let inline_qos = ParameterListSubmessageElement {
                    parameter: change.inline_qos().into_iter().collect(),
                };
                let serialized_payload = SerializedDataSubmessageElement {
                    value: change.data_value(),
                };
                let data_submessage = Data::new(
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
                );
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
                let gap_list = SequenceNumberSetSubmessageElement {
                    base: seq_num,
                    set: core::iter::empty().collect(),
                };
                let gap_submessage =
                    Gap::new(endianness_flag, reader_id, writer_id, gap_start, gap_list);
                send_gap(gap_submessage)
            }
        }
    }
}
