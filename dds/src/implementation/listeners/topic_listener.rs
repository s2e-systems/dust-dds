use crate::{
    dcps::runtime::{DdsRuntime, Spawner},
    runtime::mpsc::{mpsc_channel, MpscSender},
    topic_definition::topic_listener::TopicListener,
};

use super::domain_participant_listener::ListenerMail;

pub struct TopicListenerActor;

impl TopicListenerActor {
    pub fn spawn<R: DdsRuntime>(
        _listener: impl TopicListener<R> + Send + 'static,
        spawner_handle: &R::SpawnerHandle,
    ) -> MpscSender<ListenerMail<R>> {
        let (listener_sender, listener_receiver) = mpsc_channel();
        spawner_handle.spawn(async move { while let Some(_m) = listener_receiver.recv().await {} });
        listener_sender
    }
}
