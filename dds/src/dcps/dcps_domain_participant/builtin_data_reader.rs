use crate::infrastructure::{instance::InstanceHandle, qos::DataReaderQos};
use alloc::string::String;
use core::ops::{Deref, DerefMut};

use super::data_reader_entity::DataReaderEntity;

pub struct BuiltinDataReader<T> {
    pub reader: DataReaderEntity<T>,
}

impl<T> Deref for BuiltinDataReader<T> {
    type Target = DataReaderEntity<T>;
    fn deref(&self) -> &Self::Target {
        &self.reader
    }
}

impl<T> DerefMut for BuiltinDataReader<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader
    }
}

impl<T> BuiltinDataReader<T> {
    pub fn new(
        instance_handle: InstanceHandle,
        qos: DataReaderQos,
        topic_name: String,
        transport_reader: T,
    ) -> Self {
        Self {
            reader: DataReaderEntity::new(instance_handle, qos, topic_name, transport_reader),
        }
    }
}
