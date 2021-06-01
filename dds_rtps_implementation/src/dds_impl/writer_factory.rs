use rust_dds_api::{
    dcps_psm::StatusMask, infrastructure::qos::DataWriterQos,
    publication::data_writer_listener::DataWriterListener,
};
use rust_rtps_pim::{
    behavior::types::DurationType,
    messages::types::ParameterIdType,
    structure::types::{
        DataType, EntityIdPIM, GUIDType, GuidPrefixPIM, InstanceHandleType, LocatorType,
        ParameterListType, SequenceNumberType,
    },
};

use crate::rtps_impl::rtps_writer_impl::RTPSWriterImpl;

pub trait WriterFactoryTrait:
    GuidPrefixPIM
    + SequenceNumberType
    + EntityIdPIM
    + DurationType
    + DataType
    + LocatorType
    + InstanceHandleType
    + ParameterIdType
    + GUIDType<Self>
    + ParameterListType<Self>
    + Sized
{
}

impl<
        T: GuidPrefixPIM
            + SequenceNumberType
            + EntityIdPIM
            + DurationType
            + DataType
            + LocatorType
            + InstanceHandleType
            + ParameterIdType
            + GUIDType<Self>
            + ParameterListType<Self>
            + Sized,
    > WriterFactoryTrait for T
{
}

pub struct WriterFactory<PSM: WriterFactoryTrait> {
    guid_prefix: PSM::GuidPrefixType,
    datawriter_counter: u8,
}

impl<PSM: WriterFactoryTrait> WriterFactory<PSM> {
    pub fn new(guid_prefix: PSM::GuidPrefixType) -> Self {
        Self {
            guid_prefix,
            datawriter_counter: 0,
        }
    }

    pub fn create_datawriter<'a, T>(
        &mut self,
        _qos: DataWriterQos,
        _a_listener: Option<&'a (dyn DataWriterListener<DataType = T> + 'a)>,
        _mask: StatusMask,
    ) -> RTPSWriterImpl<PSM> {
        todo!()
    }
}
