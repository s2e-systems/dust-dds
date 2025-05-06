use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    publication::{
        data_writer_listener::DataWriterListener, publisher_listener::PublisherListener,
    },
    subscription::{
        data_reader_listener::DataReaderListener, subscriber_listener::SubscriberListener,
    },
    topic_definition::topic_listener::TopicListener,
};

/// NoOp Listener which is provided as a convinience type for the user to install as a listener when it intends to have no operation executed.
/// This object implements all the listener trait and can be used on any DDS entity.
pub struct NoOpListener;

impl DomainParticipantListener for NoOpListener {}

impl PublisherListener for NoOpListener {}

impl SubscriberListener for NoOpListener {}

impl TopicListener for NoOpListener {}

impl<'a, Foo> DataWriterListener<'a, Foo> for NoOpListener {}

impl<'a, Foo> DataReaderListener<'a, Foo> for NoOpListener {}
