use crate::{
    dcps::{
        channels::{mpsc::MpscReceiver, oneshot::OneshotSender},
        dcps_participant_factory::DcpsParticipantFactory,
        status_condition::StatusConditionEntity,
    },
    infrastructure::status::StatusKind,
    runtime::DdsRuntime,
};

impl<R: DdsRuntime, T> DcpsParticipantFactory<R, T> {
    pub fn get_status_condition_enabled_statuses(
        &mut self,
        entity: StatusConditionEntity,
        reply_sender: OneshotSender<Vec<StatusKind>>,
    ) {
        todo!()
    }

    pub fn set_status_condition_enabled_statuses(
        &mut self,
        entity: StatusConditionEntity,
        status_mask: Vec<StatusKind>,
    ) {
        todo!()
    }

    pub fn get_status_condition_trigger_value(
        &mut self,
        entity: StatusConditionEntity,
        reply_sender: OneshotSender<bool>,
    ) {
        todo!()
    }

    pub fn register_notification(
        &mut self,
        entity: StatusConditionEntity,
        reply_sender: OneshotSender<MpscReceiver<()>>,
    ) {
        todo!()
    }
}
