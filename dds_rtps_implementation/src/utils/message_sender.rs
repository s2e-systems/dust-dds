use std::{cell::RefCell, iter::FromIterator};

use rust_rtps_pim::{
    behavior::{
        stateless_writer_behavior::StatelessWriterBehavior,
        writer::reader_locator::RtpsReaderLocator,
    },
    messages::{
        submessage_elements::Parameter, submessages::RtpsSubmessageType, RtpsMessage,
        RtpsMessageHeader,
    },
    structure::{
        types::{Locator, SequenceNumber},
        RtpsEntity, RtpsParticipant,
    },
};

use crate::rtps_impl::rtps_writer_impl::RtpsWriterImpl;

use super::transport::TransportWrite;

pub trait RtpsSubmessageSender<'a, S> {
    fn create_submessages(
        &'a mut self,
    ) -> Vec<(
        Locator,
        Vec<RtpsSubmessageType<'a, S, &'a [Parameter<'a>], (), ()>>,
    )>;
}

impl<'a, S, T> RtpsSubmessageSender<'a, S> for T
where
    T: StatelessWriterBehavior<'a, S>,
    T::ReaderLocator: RtpsReaderLocator,
    S: FromIterator<SequenceNumber>,
{
    fn create_submessages(
        &'a mut self,
    ) -> Vec<(
        Locator,
        Vec<RtpsSubmessageType<'a, S, &'a [Parameter<'a>], (), ()>>,
    )> {
        let destined_submessages: Vec<(
            Locator,
            Vec<RtpsSubmessageType<'a, S, &'a [Parameter<'a>], (), ()>>,
        )> = Vec::new();
        let destined_submessages = RefCell::new(destined_submessages);
        self.send_unsent_data(
            |reader_locator, data| {
                let mut destined_submessages_borrow = destined_submessages.borrow_mut();
                match destined_submessages_borrow
                    .iter_mut()
                    .find(|(locator, _)| locator == reader_locator.locator())
                {
                    Some((_, submessages)) => submessages.push(RtpsSubmessageType::Data(data)),
                    None => destined_submessages_borrow.push((
                        *reader_locator.locator(),
                        vec![RtpsSubmessageType::Data(data)],
                    )),
                }
            },
            |reader_locator, gap| {
                let mut destined_submessages_borrow = destined_submessages.borrow_mut();
                match destined_submessages_borrow
                    .iter_mut()
                    .find(|(locator, _)| locator == reader_locator.locator())
                {
                    Some((_, submessages)) => submessages.push(RtpsSubmessageType::Gap(gap)),
                    None => destined_submessages_borrow.push((
                        *reader_locator.locator(),
                        vec![RtpsSubmessageType::Gap(gap)],
                    )),
                }
            },
        );
        destined_submessages.take()
    }
}

pub fn send_data<Transport, Participant>(
    participant: &Participant,
    writer: &mut RtpsWriterImpl,
    transport: &mut Transport,
) where
    Transport: TransportWrite,
    Participant: RtpsParticipant + RtpsEntity,
{
    let destined_submessages = writer.create_submessages();
    for (dst_locator, submessages) in destined_submessages {
        let header = RtpsMessageHeader {
            protocol: rust_rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
            version: *participant.protocol_version(),
            vendor_id: *participant.vendor_id(),
            guid_prefix: *participant.guid().prefix(),
        };
        let message = RtpsMessage {
            header,
            submessages,
        };
        transport.write(&message, &dst_locator);
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use rust_rtps_pim::{
        discovery::sedp::builtin_endpoints::ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
        messages::{
            submessage_elements::{
                EntityIdSubmessageElement, ParameterListSubmessageElement,
                SequenceNumberSetSubmessageElement, SequenceNumberSubmessageElement,
                SerializedDataSubmessageElement,
            },
            submessages::{DataSubmessage, GapSubmessage, RtpsSubmessagePIM, RtpsSubmessageType},
        },
        structure::types::{self, ENTITYID_UNKNOWN, LOCATOR_INVALID},
    };

    use super::*;

    #[derive(PartialEq, Debug)]
    struct MockPSM;

    impl<'a> RtpsSubmessagePIM<'a> for MockPSM {
        type AckNackSubmessageType = ();
        type DataSubmessageType = DataSubmessage<'a, &'a [Parameter<'a>]>;
        type DataFragSubmessageType = ();
        type GapSubmessageType = GapSubmessage<Vec<SequenceNumber>>;
        type HeartbeatSubmessageType = ();
        type HeartbeatFragSubmessageType = ();
        type InfoDestinationSubmessageType = ();
        type InfoReplySubmessageType = ();
        type InfoSourceSubmessageType = ();
        type InfoTimestampSubmessageType = ();
        type NackFragSubmessageType = ();
        type PadSubmessageType = ();
    }

    struct MockReaderLocator(Locator);

    impl RtpsReaderLocator for MockReaderLocator {
        fn locator(&self) -> &types::Locator {
            &self.0
        }

        fn expects_inline_qos(&self) -> bool {
            todo!()
        }
    }

    #[test]
    fn submessage_send_empty() {
        struct MockBehavior;

        impl<'a> StatelessWriterBehavior<'a, Vec<SequenceNumber>> for MockBehavior {
            type ReaderLocator = MockReaderLocator;

            fn send_unsent_data(
                &'a mut self,
                _send_data: impl FnMut(&Self::ReaderLocator, DataSubmessage<'a, &'a [Parameter<'a>]>),
                _send_gap: impl FnMut(&Self::ReaderLocator, GapSubmessage<Vec<SequenceNumber>>),
            ) {
            }
        }

        let mut writer = MockBehavior;
        let destined_submessages: Vec<(
            Locator,
            Vec<RtpsSubmessageType<'_, Vec<SequenceNumber>, &[Parameter], (), ()>>,
        )> = writer.create_submessages();

        assert!(destined_submessages.is_empty());
    }

    #[test]
    fn submessage_send_single_locator_send_only_data() {
        struct MockBehavior;

        const DATA_SUBMESSAGE1: DataSubmessage<&[Parameter]> = DataSubmessage {
            endianness_flag: false,
            inline_qos_flag: true,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_id: EntityIdSubmessageElement {
                value: ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            },
            writer_sn: SequenceNumberSubmessageElement { value: 1 },
            inline_qos: ParameterListSubmessageElement::<&[Parameter]> {
                parameter: &[],
                phantom: PhantomData,
            },
            serialized_payload: SerializedDataSubmessageElement { value: &[1, 2, 3] },
        };

        const DATA_SUBMESSAGE2: DataSubmessage<&[Parameter]> = DataSubmessage {
            endianness_flag: false,
            inline_qos_flag: true,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_id: EntityIdSubmessageElement {
                value: ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            },
            writer_sn: SequenceNumberSubmessageElement { value: 1 },
            inline_qos: ParameterListSubmessageElement::<&[Parameter]> {
                parameter: &[],
                phantom: PhantomData,
            },
            serialized_payload: SerializedDataSubmessageElement { value: &[4, 5, 6] },
        };

        impl<'a> StatelessWriterBehavior<'a, Vec<SequenceNumber>> for MockBehavior {
            type ReaderLocator = MockReaderLocator;

            fn send_unsent_data(
                &'a mut self,
                mut send_data: impl FnMut(&Self::ReaderLocator, DataSubmessage<'a, &'a [Parameter<'a>]>),
                _send_gap: impl FnMut(&Self::ReaderLocator, GapSubmessage<Vec<SequenceNumber>>),
            ) {
                send_data(&MockReaderLocator(LOCATOR_INVALID), DATA_SUBMESSAGE1);
                send_data(&MockReaderLocator(LOCATOR_INVALID), DATA_SUBMESSAGE2);
            }
        }

        let mut writer = MockBehavior;
        let destined_submessages: Vec<(
            Locator,
            Vec<RtpsSubmessageType<'_, Vec<SequenceNumber>, &[Parameter], (), ()>>,
        )> = writer.create_submessages();
        let (dst_locator, submessages) = &destined_submessages[0];

        assert_eq!(dst_locator, &LOCATOR_INVALID);
        assert_eq!(
            submessages,
            &vec![
                RtpsSubmessageType::Data(DATA_SUBMESSAGE1),
                RtpsSubmessageType::Data(DATA_SUBMESSAGE2)
            ]
        );
    }

    #[test]
    fn submessage_send_multiple_locator_send_data_and_gap() {
        struct MockBehavior;

        const DATA_SUBMESSAGE1: DataSubmessage<&[Parameter]> = DataSubmessage {
            endianness_flag: false,
            inline_qos_flag: true,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_id: EntityIdSubmessageElement {
                value: ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            },
            writer_sn: SequenceNumberSubmessageElement { value: 1 },
            inline_qos: ParameterListSubmessageElement::<&[Parameter]> {
                parameter: &[],
                phantom: PhantomData,
            },
            serialized_payload: SerializedDataSubmessageElement { value: &[1, 2, 3] },
        };

        const DATA_SUBMESSAGE2: DataSubmessage<&[Parameter]> = DataSubmessage {
            endianness_flag: false,
            inline_qos_flag: true,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_id: EntityIdSubmessageElement {
                value: ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
            },
            writer_sn: SequenceNumberSubmessageElement { value: 1 },
            inline_qos: ParameterListSubmessageElement::<&[Parameter]> {
                parameter: &[],
                phantom: PhantomData,
            },
            serialized_payload: SerializedDataSubmessageElement { value: &[4, 5, 6] },
        };

        const GAP_SUBMESSAGE1: GapSubmessage<Vec<SequenceNumber>> = GapSubmessage {
            endianness_flag: true,
            reader_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            gap_start: SequenceNumberSubmessageElement { value: 1 },
            gap_list: SequenceNumberSetSubmessageElement {
                base: 1,
                set: vec![],
            },
        };

        impl<'a> StatelessWriterBehavior<'a, Vec<SequenceNumber>> for MockBehavior {
            type ReaderLocator = MockReaderLocator;

            fn send_unsent_data(
                &'a mut self,
                mut send_data: impl FnMut(&Self::ReaderLocator, DataSubmessage<'a, &'a [Parameter<'a>]>),
                mut send_gap: impl FnMut(&Self::ReaderLocator, GapSubmessage<Vec<SequenceNumber>>),
            ) {
                let locator1 = Locator::new(0, 1, [0; 16]);
                let locator2 = Locator::new(0, 2, [0; 16]);
                send_data(&MockReaderLocator(locator1), DATA_SUBMESSAGE1);
                send_data(&MockReaderLocator(locator1), DATA_SUBMESSAGE2);

                send_data(&MockReaderLocator(locator2), DATA_SUBMESSAGE2);
                send_gap(&MockReaderLocator(locator1), GAP_SUBMESSAGE1);

                send_gap(&MockReaderLocator(locator2), GAP_SUBMESSAGE1);
            }
        }

        let mut writer = MockBehavior;
        let destined_submessages: Vec<(
            Locator,
            Vec<RtpsSubmessageType<'_, Vec<SequenceNumber>, &[Parameter], (), ()>>,
        )> = writer.create_submessages();

        let locator1_submessages = &destined_submessages[0].1;
        let locator2_submessages = &destined_submessages[1].1;

        assert_eq!(destined_submessages.len(), 2);

        assert_eq!(locator1_submessages.len(), 3);
        assert_eq!(
            locator1_submessages[0],
            RtpsSubmessageType::Data(DATA_SUBMESSAGE1)
        );
        assert_eq!(
            locator1_submessages[1],
            RtpsSubmessageType::Data(DATA_SUBMESSAGE2)
        );
        assert_eq!(
            locator1_submessages[2],
            RtpsSubmessageType::Gap(GAP_SUBMESSAGE1)
        );

        assert_eq!(locator2_submessages.len(), 2);
        assert_eq!(
            locator2_submessages[0],
            RtpsSubmessageType::Data(DATA_SUBMESSAGE2)
        );
        assert_eq!(
            locator2_submessages[1],
            RtpsSubmessageType::Gap(GAP_SUBMESSAGE1)
        );
    }
}
