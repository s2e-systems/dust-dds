use rust_dds_api::{
    dcps_psm::StatusMask, infrastructure::qos::DataWriterQos,
    publication::data_writer_listener::DataWriterListener,
};
use rust_rtps_pim::{
    behavior::types::DurationPIM,
    messages::{submessage_elements::ParameterListSubmessageElementPIM, types::ParameterIdPIM},
    structure::types::{
        DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM, SequenceNumberPIM,
        GUIDPIM,
    },
};

use crate::rtps_impl::rtps_writer_impl::RTPSWriterImpl;

pub trait WriterFactoryTrait:
    GuidPrefixPIM
    + SequenceNumberPIM
    + EntityIdPIM
    + DurationPIM
    + DataPIM
    + LocatorPIM
    + InstanceHandlePIM
    + ParameterIdPIM
    + GUIDPIM<Self>
    + ParameterListSubmessageElementPIM<Self>
    + Sized
{
}

impl<
        T: GuidPrefixPIM
            + SequenceNumberPIM
            + EntityIdPIM
            + DurationPIM
            + DataPIM
            + LocatorPIM
            + InstanceHandlePIM
            + ParameterIdPIM
            + GUIDPIM<Self>
            + ParameterListSubmessageElementPIM<Self>
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
        _a_listener: Option<&'a (dyn DataWriterListener<DataPIM = T> + 'a)>,
        _mask: StatusMask,
    ) -> RTPSWriterImpl<PSM> {
        todo!()
    }
}
