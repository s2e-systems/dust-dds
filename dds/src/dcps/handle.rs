use crate::infrastructure::instance::InstanceHandle;

#[derive(Default)]
pub struct InstanceHandleCounter {
    counter1: u64,
    counter2: u64,
}

impl InstanceHandleCounter {
    pub const fn new() -> Self {
        Self {
            counter1: 0,
            counter2: 0,
        }
    }

    pub fn generate_new_instance_handle(&mut self) -> InstanceHandle {
        if self.counter1 == u64::MAX {
            self.counter2 += 1;
        } else {
            self.counter1 += 1;
        }

        let counter1_bytes = self.counter1.to_ne_bytes();
        let counter2_bytes = self.counter2.to_ne_bytes();

        InstanceHandle::new([
            counter1_bytes[0],
            counter1_bytes[1],
            counter1_bytes[2],
            counter1_bytes[3],
            counter1_bytes[4],
            counter1_bytes[5],
            counter1_bytes[6],
            counter1_bytes[7],
            counter2_bytes[0],
            counter2_bytes[1],
            counter2_bytes[2],
            counter2_bytes[3],
            counter2_bytes[4],
            counter2_bytes[5],
            counter2_bytes[6],
            counter2_bytes[7],
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticipantHandle {
    value: u8,
}

impl ParticipantHandle {
    pub fn new(value: u8) -> Self {
        Self { value }
    }
}

impl From<ParticipantHandle> for InstanceHandle {
    fn from(x: ParticipantHandle) -> Self {
        InstanceHandle::new([x.value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubscriberHandle {
    participant_handle: ParticipantHandle,
    value: u8,
}

impl SubscriberHandle {
    pub fn new(participant_handle: ParticipantHandle, value: u8) -> Self {
        Self {
            participant_handle,
            value,
        }
    }
}

impl From<SubscriberHandle> for InstanceHandle {
    fn from(x: SubscriberHandle) -> Self {
        InstanceHandle::new([
            x.participant_handle.value,
            x.value,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublisherHandle {
    participant_handle: ParticipantHandle,
    value: u8,
}

impl PublisherHandle {
    pub fn new(participant_handle: ParticipantHandle, value: u8) -> Self {
        Self {
            participant_handle,
            value,
        }
    }
}

impl From<PublisherHandle> for InstanceHandle {
    fn from(x: PublisherHandle) -> Self {
        InstanceHandle::new([
            x.participant_handle.value,
            0,
            x.value,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopicHandle {
    participant_handle: ParticipantHandle,
    value: u8,
}

impl TopicHandle {
    pub fn new(participant_handle: ParticipantHandle, value: u8) -> Self {
        Self {
            participant_handle,
            value,
        }
    }
}

impl From<TopicHandle> for InstanceHandle {
    fn from(x: TopicHandle) -> Self {
        InstanceHandle::new([
            x.participant_handle.value,
            0,
            0,
            x.value,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataReaderHandle {
    subscriber_handle: SubscriberHandle,
    topic_handle: TopicHandle,
    value: u8,
}

impl DataReaderHandle {
    pub fn new(subscriber_handle: SubscriberHandle, topic_handle: TopicHandle, value: u8) -> Self {
        Self {
            subscriber_handle,
            topic_handle,
            value,
        }
    }
}

impl From<DataReaderHandle> for InstanceHandle {
    fn from(x: DataReaderHandle) -> Self {
        InstanceHandle::new([
            x.subscriber_handle.participant_handle.value,
            x.subscriber_handle.value,
            0,
            x.topic_handle.value,
            x.value,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataWriterHandle {
    publisher_handle: PublisherHandle,
    topic_handle: TopicHandle,
    value: u8,
}

impl DataWriterHandle {
    pub fn new(publisher_handle: PublisherHandle, topic_handle: TopicHandle, value: u8) -> Self {
        Self {
            publisher_handle,
            topic_handle,
            value,
        }
    }
}

impl From<DataWriterHandle> for InstanceHandle {
    fn from(x: DataWriterHandle) -> Self {
        InstanceHandle::new([
            x.publisher_handle.participant_handle.value,
            0,
            x.publisher_handle.value,
            x.topic_handle.value,
            0,
            x.value,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ])
    }
}
