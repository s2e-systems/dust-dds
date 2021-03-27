use crate::{
    behavior::RTPSWriter,
    structure::{self, RTPSHistoryCache},
    RtpsPsm,
};

pub trait RTPSReaderLocator {
    type PSM: RtpsPsm;
    type HistoryCache: RTPSHistoryCache;

    fn requested_changes(&self) -> <Self::PSM as RtpsPsm>::SequenceNumberSet;
    fn unsent_changes(&self) -> <Self::PSM as RtpsPsm>::SequenceNumberSet;
    fn next_requested_change(&mut self) -> Option<<Self::PSM as structure::Types>::SequenceNumber>;
    fn next_unsent_change(&mut self) -> Option<<Self::PSM as structure::Types>::SequenceNumber>;
    fn requested_changes_set(
        &mut self,
        req_seq_num_set: &[<Self::PSM as structure::Types>::SequenceNumber],
        writer: &RTPSWriter<Self::PSM, Self::HistoryCache>,
    );
}

// impl dyn RTPSReaderLocator {
    // fn pushing_state<
    //     'a,
    //     DataSubmessage: submessages::data_submessage::Data<
    //         EntityId = EntityIdType,
    //         SequenceNumber = SequenceNumberType,
    //         ParameterId = ParameterIdType,
    //         ParameterValue = ParameterValueType,
    //         ParameterList = ParameterListType,
    //         SerializedData = &'a [u8],
    //     >,
    //     GapSubmessage: submessages::gap_submessage::Gap<
    //         EntityId = EntityIdType,
    //         SequenceNumber = SequenceNumberType,
    //         SequenceNumberList = SequenceNumberListType,
    //     >,
    // >(
    //     &mut self,
    //     the_writer: &'a RTPSWriter<
    //         GuidPrefixType,
    //         EntityIdType,
    //         LocatorType,
    //         LocatorListType,
    //         DurationType,
    //         SequenceNumberType,
    //         InstanceHandleType,
    //         DataType,
    //         ParameterIdType,
    //         ParameterValueType,
    //         ParameterListType,
    //         HistoryCacheType,
    //     >,
    // ) -> Option<DataSubmessage> {
    //     //     // RL::can_send() is always true when this function is called
    // //     // so we don't bother making an if here
    // //     Self::transition_t4(reader_locator, writer)
    //     self.transition_t4(the_writer)
    // }

    // fn transition_t4<
    //     'a,
    //     DataSubmessage: submessages::data_submessage::Data<
    //         EntityId = EntityIdType,
    //         SequenceNumber = SequenceNumberType,
    //         ParameterId = ParameterIdType,
    //         ParameterValue = ParameterValueType,
    //         ParameterList = ParameterListType,
    //         SerializedData = &'a [u8],
    //     >,
    //     GapSubmessage: submessages::gap_submessage::Gap<
    //         EntityId = EntityIdType,
    //         SequenceNumber = SequenceNumberType,
    //         SequenceNumberList = SequenceNumberListType,
    //     >,
    // >(
    //     &mut self,
    //     the_writer: &'a RTPSWriter<
    //         GuidPrefixType,
    //         EntityIdType,
    //         LocatorType,
    //         LocatorListType,
    //         DurationType,
    //         SequenceNumberType,
    //         InstanceHandleType,
    //         DataType,
    //         ParameterIdType,
    //         ParameterValueType,
    //         ParameterListType,
    //         HistoryCacheType,
    //     >,
    // ) -> Option<DataSubmessage> {
    //     if let Some(next_unsent_sequence_number) = self.next_unsent_change() {
    //         if let Some(next_unsent_cache_change) = the_writer
    //             .writer_cache
    //             .get_change(&next_unsent_sequence_number)
    //         {
    //             Some(data_submessage_from_cache_change(
    //                 next_unsent_cache_change,
    //                 <EntityIdType as EntityId>::ENTITYID_UNKNOWN,
    //             ))
    //         } else {
    //             let gap = GapSubmessage::new(
    //                 true.into(),
    //                 submessage_elements::EntityId {
    //                     value: <EntityIdType as EntityId>::ENTITYID_UNKNOWN,
    //                 },
    //                 submessage_elements::EntityId {
    //                     value: the_writer.endpoint.guid.entity_id,
    //                 },
    //                 submessage_elements::SequenceNumber {
    //                     value: next_unsent_sequence_number,
    //                 },
    //                 submessage_elements::SequenceNumberSet {
    //                     base: next_unsent_sequence_number,
    //                     set: self.requested_changes(),
    //                 },
    //             );
    //             todo!()
    //         }
    //     } else {
    //         None
    //     }
    // }
// }
