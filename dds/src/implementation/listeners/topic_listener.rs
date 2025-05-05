use crate::{
    runtime::{
        executor::ExecutorHandle,
        mpsc::{mpsc_channel, MpscSender},
    },
    topic_definition::topic_listener::TopicListener,
};

pub struct TopicListenerActor {
    _listener: Box<dyn TopicListener + Send>,
}

impl TopicListenerActor {
    pub fn spawn(
        _listener: Box<dyn TopicListener + Send>,
        executor_handle: &ExecutorHandle,
    ) -> MpscSender<TopicListenerActorMail> {
        let (listener_sender, listener_receiver) = mpsc_channel();
        executor_handle
            .spawn(async move { while let Some(_m) = listener_receiver.recv().await {} });
        listener_sender
    }
}

pub enum TopicListenerActorMail {}
