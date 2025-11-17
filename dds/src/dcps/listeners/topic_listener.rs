use core::pin::Pin;

use crate::{
    dcps::channels::mpsc::{MpscSender, mpsc_channel},
    runtime::{DdsRuntime, Spawner},
    topic_definition::topic_listener::TopicListener,
};

use super::domain_participant_listener::ListenerMail;

pub struct DcpsTopicListener {
    sender: MpscSender<ListenerMail>,
    task: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl DcpsTopicListener {
    pub fn new(_listener: impl TopicListener + Send + 'static) -> Self {
        let (sender, listener_receiver) = mpsc_channel();
        let task =
            Box::pin(async move { while let Some(_m) = listener_receiver.receive().await {} });
        Self { sender, task }
    }

    pub fn spawn<R: DdsRuntime>(self, spawner: &R::SpawnerHandle) -> MpscSender<ListenerMail> {
        spawner.spawn(self.task);
        self.sender
    }
}
