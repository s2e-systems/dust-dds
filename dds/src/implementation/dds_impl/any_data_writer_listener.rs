use crate::{
    implementation::utils::shared_object::{DdsShared, DdsWeak},
    publication::{data_writer::DataWriter, data_writer_listener::DataWriterListener},
    topic_definition::type_support::{DdsSerialize, DdsType},
};

use super::{
    domain_participant_impl::DomainParticipantImpl,
    user_defined_data_writer::UserDefinedDataWriter, user_defined_publisher::UserDefinedPublisher,
};

pub trait AnyDataWriterListener {
    fn trigger_on_liveliness_lost(
        &mut self,
        _the_writer: &DdsShared<UserDefinedDataWriter>,
        participant: DdsWeak<DomainParticipantImpl>,
        publisher: DdsWeak<UserDefinedPublisher>,
    );
    fn trigger_on_offered_deadline_missed(
        &mut self,
        _the_writer: &DdsShared<UserDefinedDataWriter>,
        participant: DdsWeak<DomainParticipantImpl>,
        publisher: DdsWeak<UserDefinedPublisher>,
    );
    fn trigger_on_offered_incompatible_qos(
        &mut self,
        _the_writer: &DdsShared<UserDefinedDataWriter>,
        participant: DdsWeak<DomainParticipantImpl>,
        publisher: DdsWeak<UserDefinedPublisher>,
    );
    fn trigger_on_publication_matched(
        &mut self,
        _the_writer: &DdsShared<UserDefinedDataWriter>,
        participant: DdsWeak<DomainParticipantImpl>,
        publisher: DdsWeak<UserDefinedPublisher>,
    );
}

impl<Foo> AnyDataWriterListener for Box<dyn DataWriterListener<Foo = Foo> + Send + Sync>
where
    Foo: DdsType + DdsSerialize + 'static,
{
    fn trigger_on_liveliness_lost(
        &mut self,
        the_writer: &DdsShared<UserDefinedDataWriter>,
        participant: DdsWeak<DomainParticipantImpl>,
        publisher: DdsWeak<UserDefinedPublisher>,
    ) {
        self.on_liveliness_lost(
            &DataWriter::new(the_writer.downgrade(), participant, publisher),
            the_writer.get_liveliness_lost_status(),
        );
    }

    fn trigger_on_offered_deadline_missed(
        &mut self,
        the_writer: &DdsShared<UserDefinedDataWriter>,
        participant: DdsWeak<DomainParticipantImpl>,
        publisher: DdsWeak<UserDefinedPublisher>,
    ) {
        self.on_offered_deadline_missed(
            &DataWriter::new(the_writer.downgrade(), participant, publisher),
            the_writer.get_offered_deadline_missed_status(),
        );
    }

    fn trigger_on_offered_incompatible_qos(
        &mut self,
        the_writer: &DdsShared<UserDefinedDataWriter>,
        participant: DdsWeak<DomainParticipantImpl>,
        publisher: DdsWeak<UserDefinedPublisher>,
    ) {
        self.on_offered_incompatible_qos(
            &DataWriter::new(the_writer.downgrade(), participant, publisher),
            the_writer.get_offered_incompatible_qos_status(),
        );
    }

    fn trigger_on_publication_matched(
        &mut self,
        the_writer: &DdsShared<UserDefinedDataWriter>,
        participant: DdsWeak<DomainParticipantImpl>,
        publisher: DdsWeak<UserDefinedPublisher>,
    ) {
        self.on_publication_matched(
            &DataWriter::new(the_writer.downgrade(), participant, publisher),
            the_writer.get_publication_matched_status(),
        )
    }
}
