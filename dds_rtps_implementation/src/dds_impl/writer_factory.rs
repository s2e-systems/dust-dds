use std::marker::PhantomData;

use rust_dds_api::{
    dcps_psm::StatusMask, infrastructure::qos::DataWriterQos,
    publication::data_writer_listener::DataWriterListener,
};
use rust_rtps_pim::{
    behavior::types::DurationPIM, messages::submessage_elements::ParameterListSubmessageElementPIM,
    structure::types::InstanceHandlePIM,
};

use crate::rtps_impl::rtps_writer_impl::RTPSWriterImpl;

pub struct WriterFactory<PSM> {
    guid_prefix: rust_rtps_pim::structure::types::GuidPrefix,
    datawriter_counter: u8,
    phantom: PhantomData<PSM>,
}

impl<PSM> WriterFactory<PSM> {
    pub fn new(guid_prefix: rust_rtps_pim::structure::types::GuidPrefix) -> Self {
        Self {
            guid_prefix,
            datawriter_counter: 0,
            phantom: PhantomData,
        }
    }

    pub fn create_datawriter<'a, T>(
        &mut self,
        _qos: DataWriterQos,
        _a_listener: Option<&'a (dyn DataWriterListener<DataPIM = T> + 'a)>,
        _mask: StatusMask,
    ) -> RTPSWriterImpl<PSM>
    where
        PSM: DurationPIM + InstanceHandlePIM + ParameterListSubmessageElementPIM,
    {
        todo!()
    }
}
