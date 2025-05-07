use crate::{
    runtime::{
        executor::ExecutorHandle,
        mpsc::{mpsc_channel, MpscSender},
    },
    topic_definition::topic_listener::TopicListener,
};

use super::domain_participant_listener::ListenerMail;

pub struct TopicListenerActor;

impl TopicListenerActor {
    pub fn spawn(
        _listener: impl TopicListener + Send + 'static,
        executor_handle: &ExecutorHandle,
    ) -> MpscSender<ListenerMail> {
        let (listener_sender, listener_receiver) = mpsc_channel();
        executor_handle
            .spawn(async move { while let Some(_m) = listener_receiver.recv().await {} });
        listener_sender
    }
}
