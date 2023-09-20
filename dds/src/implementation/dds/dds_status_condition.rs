use crate::{
    implementation::utils::actor::{Mail, MailHandler},
    infrastructure::status::StatusKind,
};

pub struct DdsStatusCondition {
    enabled_statuses: Vec<StatusKind>,
    status_changes: Vec<StatusKind>,
}

impl Default for DdsStatusCondition {
    fn default() -> Self {
        Self {
            enabled_statuses: vec![
                StatusKind::InconsistentTopic,
                StatusKind::OfferedDeadlineMissed,
                StatusKind::RequestedDeadlineMissed,
                StatusKind::OfferedIncompatibleQos,
                StatusKind::RequestedIncompatibleQos,
                StatusKind::SampleLost,
                StatusKind::SampleRejected,
                StatusKind::DataOnReaders,
                StatusKind::DataAvailable,
                StatusKind::LivelinessLost,
                StatusKind::LivelinessChanged,
                StatusKind::PublicationMatched,
                StatusKind::SubscriptionMatched,
            ],
            status_changes: Vec::new(),
        }
    }
}

pub struct AddCommunicationState {
    state: StatusKind,
}

impl AddCommunicationState {
    pub fn new(state: StatusKind) -> Self {
        Self { state }
    }
}

impl Mail for AddCommunicationState {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<AddCommunicationState> for DdsStatusCondition {
    async fn handle(
        &mut self,
        mail: AddCommunicationState,
    ) -> <AddCommunicationState as Mail>::Result {
        self.status_changes.push(mail.state);
    }
}

pub struct RemoveCommunicationState {
    state: StatusKind,
}

impl RemoveCommunicationState {
    pub fn new(state: StatusKind) -> Self {
        Self { state }
    }
}

impl Mail for RemoveCommunicationState {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<RemoveCommunicationState> for DdsStatusCondition {
    async fn handle(
        &mut self,
        mail: RemoveCommunicationState,
    ) -> <RemoveCommunicationState as Mail>::Result {
        self.status_changes.retain(|x| x != &mail.state);
    }
}

pub struct GetEnabledStatuses;

impl Mail for GetEnabledStatuses {
    type Result = Vec<StatusKind>;
}

#[async_trait::async_trait]
impl MailHandler<GetEnabledStatuses> for DdsStatusCondition {
    async fn handle(&mut self, _mail: GetEnabledStatuses) -> <GetEnabledStatuses as Mail>::Result {
        self.enabled_statuses.clone()
    }
}

pub struct SetEnabledStatuses {
    mask: Vec<StatusKind>,
}

impl SetEnabledStatuses {
    pub fn new(mask: Vec<StatusKind>) -> Self {
        Self { mask }
    }
}

impl Mail for SetEnabledStatuses {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<SetEnabledStatuses> for DdsStatusCondition {
    async fn handle(&mut self, mail: SetEnabledStatuses) -> <SetEnabledStatuses as Mail>::Result {
        self.enabled_statuses = mail.mask;
    }
}

pub struct GetTriggerValue;

impl Mail for GetTriggerValue {
    type Result = bool;
}

#[async_trait::async_trait]
impl MailHandler<GetTriggerValue> for DdsStatusCondition {
    async fn handle(&mut self, _mail: GetTriggerValue) -> <GetTriggerValue as Mail>::Result {
        for status in &self.status_changes {
            if self.enabled_statuses.contains(status) {
                return true;
            }
        }
        false
    }
}
