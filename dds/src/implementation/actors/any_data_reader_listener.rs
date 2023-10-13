use crate::{
    implementation::utils::actor::ActorAddress,
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    subscription::{data_reader::DataReader, data_reader_listener::DataReaderListener},
};

use super::{
    data_reader_actor::DataReaderActor, domain_participant_actor::DomainParticipantActor,
    subscriber_actor::SubscriberActor,
};

pub trait AnyDataReaderListener {
    fn trigger_on_data_available(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
    );
    fn trigger_on_sample_rejected(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: SampleRejectedStatus,
    );
    fn trigger_on_liveliness_changed(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: LivelinessChangedStatus,
    );
    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: RequestedDeadlineMissedStatus,
    );
    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: RequestedIncompatibleQosStatus,
    );
    fn trigger_on_subscription_matched(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: SubscriptionMatchedStatus,
    );
    fn trigger_on_sample_lost(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: SampleLostStatus,
    );
}

impl<Foo> AnyDataReaderListener for Box<dyn DataReaderListener<Foo> + Send> {
    fn trigger_on_data_available(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) {
        self.on_data_available(&DataReader::new(
            reader_address,
            subscriber_address,
            participant_address,
        ))
    }

    fn trigger_on_sample_rejected(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: SampleRejectedStatus,
    ) {
        self.on_sample_rejected(
            &DataReader::new(reader_address, subscriber_address, participant_address),
            status,
        )
    }

    fn trigger_on_liveliness_changed(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: LivelinessChangedStatus,
    ) {
        self.on_liveliness_changed(
            &DataReader::new(reader_address, subscriber_address, participant_address),
            status,
        )
    }

    fn trigger_on_requested_deadline_missed(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: RequestedDeadlineMissedStatus,
    ) {
        self.on_requested_deadline_missed(
            &DataReader::new(reader_address, subscriber_address, participant_address),
            status,
        )
    }

    fn trigger_on_requested_incompatible_qos(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: RequestedIncompatibleQosStatus,
    ) {
        self.on_requested_incompatible_qos(
            &DataReader::new(reader_address, subscriber_address, participant_address),
            status,
        )
    }

    fn trigger_on_subscription_matched(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: SubscriptionMatchedStatus,
    ) {
        self.on_subscription_matched(
            &DataReader::new(reader_address, subscriber_address, participant_address),
            status,
        )
    }

    fn trigger_on_sample_lost(
        &mut self,
        reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: SampleLostStatus,
    ) {
        self.on_sample_lost(
            &DataReader::new(reader_address, subscriber_address, participant_address),
            status,
        )
    }
}
