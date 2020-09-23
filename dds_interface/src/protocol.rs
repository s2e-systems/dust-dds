use crate::types::InstanceHandle;

pub trait WriterInterface{
    fn new() -> Self;
    
    fn write(&self, instance_handle: InstanceHandle);

    fn dispose(&self);

    fn unregister(&self);

    fn register(&self);
}

pub trait ReaderProtocolInterface {

}