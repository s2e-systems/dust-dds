use core::{
    iter::FromIterator,
    ops::{Deref, DerefMut},
};

use crate::{
    behavior::types::Duration,
    messages::submessages::{DataSubmessage, GapSubmessage},
    structure::{
        types::{Guid, ReliabilityKind, SequenceNumber, TopicKind},
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

pub trait StatefulWriterBehavior<S, P, D> {
    type ReaderProxy;

    fn send_unsent_data(
        &mut self,
        send_data: impl FnMut(&Self::ReaderProxy, DataSubmessage<P, D>),
        send_gap: impl FnMut(&Self::ReaderProxy, GapSubmessage<S>),
    );
}

impl<S, L, C, R, RP, P, D> StatefulWriterBehavior<S, P, D> for RtpsStatefulWriter<L, C, R>
where
    for<'b> &'b mut R: IntoIterator<Item = &'b mut RP>,
    RP: RtpsReaderProxyOperations,
    C: for<'a> RtpsHistoryCacheOperations<'a, GetChangeDataType = D, GetChangeParameterType = P>,
    S: FromIterator<SequenceNumber>,
{
    type ReaderProxy = RP;

    fn send_unsent_data(
        &mut self,
        mut send_data: impl FnMut(&Self::ReaderProxy, DataSubmessage<P, D>),
        mut send_gap: impl FnMut(&Self::ReaderProxy, GapSubmessage<S>),
    ) {
        let reliability_level = self.writer.reliability_level;
        let last_change_sequence_number = self.writer.last_change_sequence_number;
        for reader_locator in &mut self.matched_readers {
            match reliability_level {
                ReliabilityKind::BestEffort => todo!(),
                ReliabilityKind::Reliable => todo!(),
            }
        }
    }
}

// fn best_effort_send_unsent_data<RL, S, P, D>(
//     reader_locator: &mut RL,
//     writer_cache: &impl for<'a> RtpsHistoryCacheOperations<
//         'a,
//         GetChangeDataType = D,
//         GetChangeParameterType = P,
//     >,
//     last_change_sequence_number: &SequenceNumber,
//     send_data: &mut impl FnMut(&RL, DataSubmessage<P, D>),
//     send_gap: &mut impl FnMut(&RL, GapSubmessage<S>),
// ) where
//     RL: RtpsReaderLocatorOperations,
//     S: FromIterator<SequenceNumber>,
// {
//     while let Some(seq_num) = reader_locator.next_unsent_change(&last_change_sequence_number) {
//         if let Some(change) = writer_cache.get_change(&seq_num) {
//             let endianness_flag = true;
//             let inline_qos_flag = true;
//             let (data_flag, key_flag) = match change.kind {
//                 ChangeKind::Alive => (true, false),
//                 ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => (false, true),
//                 _ => todo!(),
//             };
//             let non_standard_payload_flag = false;
//             let reader_id = EntityIdSubmessageElement {
//                 value: ENTITYID_UNKNOWN,
//             };
//             let writer_id = EntityIdSubmessageElement {
//                 value: *change.writer_guid.entity_id(),
//             };
//             let writer_sn = SequenceNumberSubmessageElement {
//                 value: change.sequence_number,
//             };
//             let inline_qos = ParameterListSubmessageElement {
//                 parameter: change.inline_qos,
//             };
//             let serialized_payload = SerializedDataSubmessageElement {
//                 value: change.data_value,
//             };
//             let data_submessage = DataSubmessage {
//                 endianness_flag,
//                 inline_qos_flag,
//                 data_flag,
//                 key_flag,
//                 non_standard_payload_flag,
//                 reader_id,
//                 writer_id,
//                 writer_sn,
//                 inline_qos,
//                 serialized_payload,
//             };
//             send_data(reader_locator, data_submessage)
//         } else {
//             let endianness_flag = true;
//             let reader_id = EntityIdSubmessageElement {
//                 value: ENTITYID_UNKNOWN,
//             };
//             let writer_id = EntityIdSubmessageElement {
//                 value: ENTITYID_UNKNOWN,
//             };
//             let gap_start = SequenceNumberSubmessageElement { value: seq_num };
//             let set = core::iter::empty().collect();
//             let gap_list = SequenceNumberSetSubmessageElement { base: seq_num, set };
//             let gap_submessage = GapSubmessage {
//                 endianness_flag,
//                 reader_id,
//                 writer_id,
//                 gap_start,
//                 gap_list,
//             };
//             send_gap(reader_locator, gap_submessage)
//         }
//     }
// }

// pub fn reliable_send_unsent_data(
//     reader_locator: &mut impl RtpsReaderLocatorOperations,
//     last_change_sequence_number: SequenceNumber,
//     mut send: impl FnMut(SequenceNumber),
// ) {
//     while let Some(seq_num) = reader_locator.next_unsent_change(&last_change_sequence_number) {
//         send(seq_num)
//     }
// }
