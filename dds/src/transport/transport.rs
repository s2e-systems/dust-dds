use super::{
    reader::{ReaderHistoryCache, TransportReader},
    types::TopicKind,
    writer::WriterHistoryCache,
};

pub trait Transport: Send + Sync {
    fn guid(&self) -> [u8; 16];

    fn get_participant_discovery_writer(&self) -> Box<dyn WriterHistoryCache>;

    fn get_participant_discovery_reader(&self) -> Box<dyn TransportReader>;

    fn get_topics_discovery_writer(&self) -> Box<dyn WriterHistoryCache>;

    fn get_topics_discovery_reader(&self) -> Box<dyn TransportReader>;

    fn get_publications_discovery_writer(&self) -> Box<dyn WriterHistoryCache>;

    fn get_publications_discovery_reader(&self) -> Box<dyn TransportReader>;

    fn get_subscriptions_discovery_writer(&self) -> Box<dyn WriterHistoryCache>;

    fn get_subscriptions_discovery_reader(&self) -> Box<dyn TransportReader>;

    fn create_user_defined_reader(
        &mut self,
        topic_name: &str,
        topic_kind: TopicKind,
        reader_history_cache: Box<dyn ReaderHistoryCache>,
    ) -> Box<dyn TransportReader>;

    fn create_user_defined_writer(
        &mut self,
        topic_name: &str,
        topic_kind: TopicKind,
    ) -> Box<dyn WriterHistoryCache>;
}
