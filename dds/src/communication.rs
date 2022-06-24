use dds_implementation::{
    dds_impl::{
        publisher_attributes::PublisherAttributes, subscriber_attributes::SubscriberAttributes,
    },
    utils::{rtps_communication_traits::SendRtpsMessage, shared_object::DdsShared},
};

use rtps_pim::{
    messages::{
        overall_structure::RtpsSubmessageType, submessage_elements::Parameter,
        types::FragmentNumber,
    },
    structure::types::{GuidPrefix, Locator, ProtocolVersion, SequenceNumber, VendorId},
    transport::{TransportRead, TransportWrite},
};

use crate::{domain_participant_factory::RtpsStructureImpl, message_receiver::MessageReceiver};

pub struct Communication<T> {
    pub version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
    pub transport: T,
}

impl<T> Communication<T>
where
    T: for<'a> TransportWrite<
        Vec<
            RtpsSubmessageType<
                Vec<SequenceNumber>,
                Vec<Parameter<'a>>,
                &'a [u8],
                Vec<Locator>,
                Vec<FragmentNumber>,
            >,
        >,
    >,
{
    pub fn send_publisher_message(
        &mut self,
        publisher: DdsShared<PublisherAttributes<RtpsStructureImpl>>,
    ) {
        publisher.send_message(&mut self.transport);
    }

    pub fn send_subscriber_message(
        &mut self,
        subscriber: DdsShared<SubscriberAttributes<RtpsStructureImpl>>,
    ) {
        subscriber.send_message(&mut self.transport);
    }
}

impl<T> Communication<T>
where
    T: for<'a> TransportRead<
        'a,
        Vec<
            RtpsSubmessageType<
                Vec<SequenceNumber>,
                Vec<Parameter<'a>>,
                &'a [u8],
                Vec<Locator>,
                Vec<FragmentNumber>,
            >,
        >,
    >,
{
    pub fn receive(
        &mut self,
        publisher_list: &[DdsShared<PublisherAttributes<RtpsStructureImpl>>],
        subscriber_list: &[DdsShared<SubscriberAttributes<RtpsStructureImpl>>],
    ) {
        while let Some((source_locator, message)) = self.transport.read() {
            MessageReceiver::new().process_message(
                self.guid_prefix,
                publisher_list,
                subscriber_list,
                source_locator,
                &message,
            );
        }
    }
}
