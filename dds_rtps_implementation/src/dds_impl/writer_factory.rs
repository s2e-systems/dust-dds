use rust_dds_api::{
    dcps_psm::StatusMask, infrastructure::qos::DataWriterQos,
    publication::data_writer_listener::DataWriterListener,
};
use rust_rtps_pim::{
    behavior::types::DurationPIM,
    messages::submessage_elements::ParameterListSubmessageElementPIM,
    structure::types::{
        DataPIM, EntityIdPIM, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM, SequenceNumberPIM,
        GUIDPIM,
    },
};

use crate::rtps_impl::rtps_writer_impl::RTPSWriterImpl;

pub struct WriterFactory<PSM>
where
    PSM: GuidPrefixPIM,
{
    guid_prefix: PSM::GuidPrefixType,
    datawriter_counter: u8,
}

impl<PSM> WriterFactory<PSM>
where
    PSM: GuidPrefixPIM,
{
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
    ) -> RTPSWriterImpl<PSM>
    where
        PSM: GUIDPIM
            + LocatorPIM
            + DurationPIM
            + SequenceNumberPIM
            + EntityIdPIM
            + InstanceHandlePIM
            + DataPIM
            + ParameterListSubmessageElementPIM,
    {
        todo!()
    }
}
