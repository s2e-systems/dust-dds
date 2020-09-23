use crate::types::{InstanceHandle, Data};

pub trait WriterInterface{
    fn new() -> Self;
    
    fn write(&self, instance_handle: InstanceHandle, data: Data);

    fn dispose(&self, instance_handle: InstanceHandle);

    fn unregister(&self, instance_handle: InstanceHandle);

    fn register(&self, instance_handle: InstanceHandle);
}

pub trait ReaderProtocolInterface {

}