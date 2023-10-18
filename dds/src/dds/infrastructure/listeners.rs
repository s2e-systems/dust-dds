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

pub struct NoListener;

impl DomainParticipantListener for NoListener {}
impl PublisherListener for NoListener {}
impl SubscriberListener for NoListener {}
impl TopicListener for NoListener {}
impl<Foo> DataReaderListener<Foo> for NoListener {}
impl<Foo> DataWriterListener<Foo> for NoListener {}
