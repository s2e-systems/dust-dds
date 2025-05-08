use crate::{
    dcps::runtime::{ChannelReceive, DdsRuntime, Spawner},
    topic_definition::topic_listener::TopicListener,
};

use super::domain_participant_listener::ListenerMail;

pub struct TopicListenerActor;

impl TopicListenerActor {
    pub fn spawn<R: DdsRuntime>(
        _listener: impl TopicListener<R> + Send + 'static,
        spawner_handle: &R::SpawnerHandle,
    ) -> R::ChannelSender<ListenerMail<R>> {
        let (listener_sender, mut listener_receiver) = R::channel();
        spawner_handle
            .spawn(async move { while let Some(_m) = listener_receiver.receive().await {} });
        listener_sender
    }
}
