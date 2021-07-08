use rust_rtps_pim::{behavior::stateless_writer::{BestEffortBehavior, RTPSReaderLocator}, messages::{
        submessage_elements::{
            EntityIdSubmessageElementPIM, EntityIdSubmessageElementType,
            ParameterListSubmessageElementPIM, SequenceNumberSetSubmessageElementPIM,
            SequenceNumberSetSubmessageElementType, SequenceNumberSubmessageElementPIM,
            SequenceNumberSubmessageElementType,
        },
        submessages::{
            AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessagePIM, GapSubmessage,
            GapSubmessagePIM, HeartbeatFragSubmessagePIM, HeartbeatSubmessagePIM,
            InfoDestinationSubmessagePIM, InfoReplySubmessagePIM, InfoSourceSubmessagePIM,
            InfoTimestampSubmessagePIM, NackFragSubmessagePIM, PadSubmessagePIM,
            RtpsSubmessageType,
        },
        RTPSMessage,
    }, structure::{RTPSCacheChange, RTPSHistoryCache, types::{Locator, SequenceNumber}}};

use crate::transport::TransportWrite;

pub fn send_data<HistoryCache, ReaderLocator, Transport>(
    writer_cache: &HistoryCache,
    reader_locators: &mut [ReaderLocator],
    last_change_sequence_number: SequenceNumber,
    transport: &mut Transport,
    header: &<<Transport as  TransportWrite>::RTPSMessageType as RTPSMessage>::RtpsMessageHeaderType,
) where
    HistoryCache: RTPSHistoryCache,
    <HistoryCache as rust_rtps_pim::structure::RTPSHistoryCache>::CacheChange: RTPSCacheChange,
    ReaderLocator: BestEffortBehavior,
    Transport: TransportWrite,
{
    for reader_locator in reader_locators {
        let mut data_submessage_list: Vec<
            RtpsSubmessageType<
                <<Transport as TransportWrite>::RTPSMessageType as RTPSMessage>::PSM,
            >,
        > = vec![];
        let mut gap_submessage_list: Vec<
            RtpsSubmessageType<
                <<Transport as TransportWrite>::RTPSMessageType as RTPSMessage>::PSM,
            >,
        > = vec![];

        let mut submessages = vec![];

        reader_locator.best_effort_send_unsent_data(
                                &last_change_sequence_number,
                                writer_cache,
                                |data_submessage| {
                                    data_submessage_list
                                        .push(RtpsSubmessageType::Data(data_submessage))
                                },
                                |gap_submessage: <<<Transport as  TransportWrite>::RTPSMessageType as RTPSMessage>::PSM as GapSubmessagePIM>::GapSubmessageType| {
                                    gap_submessage_list
                                        .push(RtpsSubmessageType::Gap(gap_submessage))
                                },
                            );

        for data_submessage in data_submessage_list {
            submessages.push(data_submessage)
        }
        for gap_submessage in gap_submessage_list {
            submessages.push(gap_submessage);
        }

        // let protocol_version = rtps_participant.protocol_version();
        // let vendor_id = rtps_participant.vendor_id();
        // let guid_prefix = rtps_participant.guid().prefix();
        // let header = <<Transport as  TransportWrite>::RTPSMessageType as RTPSMessage>::RtpsMessageHeaderType::new(protocol_version, vendor_id, guid_prefix);

        let message = Transport::RTPSMessageType::new(header, submessages);
        let destination_locator = Locator::new([0; 4], [1; 4], [0; 16]);
        transport.write(&message, &destination_locator);
    }
}

#[cfg(test)]
mod tests {}
