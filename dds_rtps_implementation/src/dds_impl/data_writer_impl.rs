use rust_dds_api::{
    builtin_topics::SubscriptionBuiltinTopicData,
    dcps_psm::{
        Duration, InstanceHandle, LivelinessLostStatus, OfferedDeadlineMissedStatus,
        OfferedIncompatibleQosStatus, PublicationMatchedStatus, StatusMask, Time,
    },
    infrastructure::{entity::StatusCondition, qos::DataWriterQos},
    publication::data_writer_listener::DataWriterListener,
    return_type::DDSResult,
};
use rust_rtps_pim::{behavior::RTPSWriter, structure::types::ChangeKind};

use crate::{
    dds_type::DDSType, rtps_impl::rtps_writer_impl::RTPSWriterImpl, utils::shared_object::RtpsWeak,
};

use super::{publisher_impl::PublisherImpl, topic_impl::TopicImpl};

pub struct DataWriterImpl<
    'dw,
    'p: 'dw,
    't: 'dw,
    T: DDSType<PSM> + 'static,
    PSM: crate::rtps_impl::PIM,
> {
    publisher: &'dw PublisherImpl<'p, PSM>,
    topic: &'dw TopicImpl<'t, T, PSM>,
    rtps_writer_impl: RtpsWeak<RTPSWriterImpl<PSM>>,
}

impl<'dw, 'p: 'dw, 't: 'dw, T: DDSType<PSM> + 't, PSM: crate::rtps_impl::PIM>
    DataWriterImpl<'dw, 'p, 't, T, PSM>
{
    pub fn new(
        publisher: &'dw PublisherImpl<'p, PSM>,
        topic: &'dw TopicImpl<'t, T, PSM>,
        rtps_writer_impl: RtpsWeak<RTPSWriterImpl<PSM>>,
    ) -> Self {
        Self {
            publisher,
            topic,
            rtps_writer_impl,
        }
    }
}

impl<'dw, 'p: 'dw, 't: 'dw, T: DDSType<PSM> + 'static, PSM: crate::rtps_impl::PIM>
    rust_dds_api::publication::data_writer::DataWriterParent
    for DataWriterImpl<'dw, 'p, 't, T, PSM>
{
    type PublisherType = PublisherImpl<'p, PSM>;

    fn get_publisher(&self) -> &Self::PublisherType {
        self.publisher
    }
}

impl<'dw, 'p: 'dw, 't: 'dw, T: DDSType<PSM> + 'static, PSM: crate::rtps_impl::PIM>
    rust_dds_api::publication::data_writer::DataWriter<T>
    for DataWriterImpl<'dw, 'p, 't, T, PSM>
{
    fn register_instance(&self, _instance: T) -> DDSResult<Option<InstanceHandle>> {
        todo!()
        // let timestamp = self.parent.0.parent.get_current_time()?;
        // self.register_instance_w_timestamp(instance, timestamp)
    }

    fn register_instance_w_timestamp(
        &self,
        _instance: T,
        _timestamp: Time,
    ) -> DDSResult<Option<InstanceHandle>> {
        // let writer = self
        //     .rtps_writer_impl
        //     .upgrade()
        //     .ok_or(DDSError::AlreadyDeleted)?;
        // let writer_guard = writer.lock().unwrap();
        // let _c = writer_guard.writer_cache();
        todo!()
    }

    fn unregister_instance(&self, _instance: T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        todo!()
    }

    fn unregister_instance_w_timestamp(
        &self,
        _instance: T,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_key_value(&self, _key_holder: &mut T, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn lookup_instance(&self, _instance: &T) -> DDSResult<Option<InstanceHandle>> {
        todo!()
    }

    fn write(&self, _data: T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        todo!()
    }

    fn write_w_timestamp(
        &self,
        data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DDSResult<()> {
        let writer = self.rtps_writer_impl.upgrade()?;
        let mut writer_lock = writer.lock();
        let change = writer_lock.new_change(ChangeKind::Alive, data.serialize(), &[], data.key());
        writer_lock.writer_cache_mut().add_change(change);
        Ok(())
    }

    fn dispose(&self, _data: T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        todo!()
    }

    fn dispose_w_timestamp(
        &self,
        _data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DDSResult<()> {
        todo!()
    }

    fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DDSResult<()> {
        todo!()
    }

    fn get_liveliness_lost_status(&self, _status: &mut LivelinessLostStatus) -> DDSResult<()> {
        todo!()
    }

    fn get_offered_deadline_missed_status(
        &self,
        _status: &mut OfferedDeadlineMissedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_offered_incompatible_qos_status(
        &self,
        _status: &mut OfferedIncompatibleQosStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_publication_matched_status(
        &self,
        _status: &mut PublicationMatchedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn assert_liveliness(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_matched_subscription_data(
        &self,
        _subscription_data: SubscriptionBuiltinTopicData,
        _subscription_handle: InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_matched_subscriptions(
        &self,
        _subscription_handles: &mut [InstanceHandle],
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<'dw, 'p: 'dw, 't: 'dw, T: DDSType<PSM> + 'static, PSM: crate::rtps_impl::PIM>
    rust_dds_api::infrastructure::entity::Entity for DataWriterImpl<'dw, 'p, 't, T, PSM>
{
    type Qos = DataWriterQos;
    type Listener = &'static dyn DataWriterListener<DataType = T>;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> StatusMask {
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
    }
}

impl<'dw, 'p: 'dw, 't: 'dw, T: DDSType<PSM> + 't, PSM: crate::rtps_impl::PIM>
    rust_dds_api::publication::data_writer::AnyDataWriter
    for DataWriterImpl<'dw, 'p, 't, T, PSM>
{
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::{
    //     dds_impl::domain_participant_impl::DomainParticipantImpl,
    //     rtps_impl::rtps_participant_impl::RTPSParticipantImpl,
    // };
    // use rust_dds_api::{
    //     domain::domain_participant::DomainParticipant,
    //     publication::{data_writer::DataWriter, publisher::Publisher},
    // };
    // use rust_rtps_udp_psm::RtpsUdpPsm;

    struct MockData;

    // impl DDSType for MockData {
    //     fn type_name() -> &'static str {
    //         todo!()
    //     }

    //     fn has_key() -> bool {
    //         todo!()
    //     }

    //     fn key(&self) -> Vec<u8> {
    //         todo!()
    //     }

    //     fn serialize(&self) -> Vec<u8> {
    //         todo!()
    //     }

    //     fn deserialize(_data: Vec<u8>) -> Self {
    //         todo!()
    //     }
    // }

    #[test]
    fn write_w_timestamp() {
        // let domain_participant: DomainParticipantImpl<RtpsUdpPsm> =
        //     DomainParticipantImpl::new(RTPSParticipantImpl::new([1; 12]));
        // let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
        // let a_topic = domain_participant
        //     .create_topic::<MockData>("Test", None, None, 0)
        //     .unwrap();

        // let data_writer = publisher
        //     .create_datawriter(&a_topic, None, None, 0)
        //     .unwrap();

        // data_writer
        //     .write_w_timestamp(MockData, None, Time { sec: 0, nanosec: 0 })
        //     .unwrap();

        // assert!(data_writer
        //     .rtps_writer
        //     .upgrade()
        //     .unwrap()
        //     .lock()
        //     .unwrap()
        //     .writer_cache()
        //     .get_change(&(1i64.into()))
        //     .is_some());
    }
}
