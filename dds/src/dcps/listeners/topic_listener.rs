use crate::{
    dcps::channels::mpsc::{MpscSender, mpsc_channel},
    runtime::{DdsRuntime, Spawner},
    topic_definition::topic_listener::TopicListener,
};

use super::domain_participant_listener::ListenerMail;

pub struct DcpsTopicListener;

impl DcpsTopicListener {
    pub fn spawn<R: DdsRuntime>(
        _listener: impl TopicListener<R> + Send + 'static,
        spawner_handle: &R::SpawnerHandle,
    ) -> MpscSender<ListenerMail<R>> {
        let (listener_sender, listener_receiver) = mpsc_channel();
        spawner_handle
            .spawn(async move { while let Some(_m) = listener_receiver.receive().await {} });
        listener_sender
    }
}
