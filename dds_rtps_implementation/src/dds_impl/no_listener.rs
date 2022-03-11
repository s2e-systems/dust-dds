use rust_dds_api::{
    domain::domain_participant_listener::DomainParticipantListener,
    publication::{
        data_writer_listener::DataWriterListener, publisher_listener::PublisherListener,
    },
    subscription::{
        data_reader_listener::DataReaderListener, subscriber_listener::SubscriberListener,
    }, topic::topic_listener::TopicListener,
};

pub struct NoListener;

impl DomainParticipantListener for NoListener {}

impl SubscriberListener for NoListener {}

impl PublisherListener for NoListener {}

impl TopicListener for NoListener {}

impl DataWriterListener for NoListener {}

impl DataReaderListener for NoListener {}