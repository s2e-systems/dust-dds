use super::{
    history_cache::HistoryCache,
    reader::TransportReader,
    types::{Guid, TopicKind},
    writer::TransportWriter,
};

pub trait TransportParticipant: Send + Sync {
    fn guid(&self) -> Guid;

    fn get_participant_discovery_writer(&self) -> Box<dyn TransportWriter>;

    fn get_participant_discovery_reader(&self) -> Box<dyn TransportReader>;

    fn get_topics_discovery_writer(&self) -> Box<dyn TransportWriter>;

    fn get_topics_discovery_reader(&self) -> Box<dyn TransportReader>;

    fn get_publications_discovery_writer(&self) -> Box<dyn TransportWriter>;

    fn get_publications_discovery_reader(&self) -> Box<dyn TransportReader>;

    fn get_subscriptions_discovery_writer(&self) -> Box<dyn TransportWriter>;

    fn get_subscriptions_discovery_reader(&self) -> Box<dyn TransportReader>;

    fn create_reader(
        &mut self,
        guid: Guid,
        topic_name: &str,
        topic_kind: TopicKind,
        reader_history_cache: Box<dyn HistoryCache>,
    ) -> Box<dyn TransportReader>;

    fn create_writer(
        &mut self,
        guid: Guid,
        topic_name: &str,
        topic_kind: TopicKind,
    ) -> Box<dyn TransportWriter>;
}
