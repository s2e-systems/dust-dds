use crate::{
    behavior::{self, RTPSWriter},
    structure::{self, types::GUID},
};

pub struct RTPSReaderProxy<PSM: structure::Types> {
    remote_reader_guid: GUID<PSM>,
    remote_group_entity_id: PSM::EntityId,
    unicast_locator_list: PSM::LocatorVector,
    multicast_locator_list: PSM::LocatorVector,
    expects_inline_qos: bool,
    is_active: bool,
}

impl<PSM: structure::Types> RTPSReaderProxy<PSM> {
    pub fn new(
        remote_reader_guid: GUID<PSM>,
        remote_group_entity_id: PSM::EntityId,
        unicast_locator_list: PSM::LocatorVector,
        multicast_locator_list: PSM::LocatorVector,
        expects_inline_qos: bool,
        is_active: bool,
    ) -> Self {
        Self {
            remote_reader_guid,
            remote_group_entity_id,
            unicast_locator_list,
            multicast_locator_list,
            expects_inline_qos,
            is_active,
        }
    }

    pub fn remote_reader_guid(&self) -> GUID<PSM> {
        self.remote_reader_guid
    }

    pub fn remote_group_entity_id(&self) -> PSM::EntityId {
        self.remote_group_entity_id
    }

    pub fn unicast_locator_list(&self) -> &PSM::LocatorVector {
        &self.unicast_locator_list
    }

    pub fn multicast_locator_list(&self) -> &PSM::LocatorVector {
        &self.multicast_locator_list
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }


    // fn acked_changes_set(
    //     &mut self,
    //     committed_seq_num: <<<Self::Writer as RTPSWriter>::HistoryCache as RTPSHistoryCache>::CacheChange as RTPSCacheChange>::SequenceNumber,
    //     writer: &Self::Writer,
    // );

    // fn next_requested_change(&mut self, writer: &Self::Writer)
    //     -> Option<Self::ChangeForReaderType>;
    // fn next_unsent_change(&mut self, writer: &Self::Writer) -> Option<Self::ChangeForReaderType>;
    // fn unsent_changes(&self, writer: &Self::Writer) -> Self::ChangeForReaderTypeList;
    // fn requested_changes(&self, writer: &Self::Writer) -> Self::ChangeForReaderTypeList;
    // fn requested_changes_set(
    //     &mut self,
    //     req_seq_num_set: &[<<<Self::Writer as RTPSWriter>::HistoryCache as RTPSHistoryCache>::CacheChange as RTPSCacheChange>::SequenceNumber],
    //     writer: &Self::Writer,
    // );
    // fn unacked_changes(&self, writer: &Self::Writer) -> Self::ChangeForReaderTypeList;
    // fn changes_for_reader(&self, writer: &Self::Writer) -> Self::ChangeForReaderTypeList;
}

pub trait RTPSStatefulWriter<PSM: structure::Types + behavior::Types>: RTPSWriter<PSM> {
    fn matched_readers(&self) -> &[RTPSReaderProxy<PSM>];
    fn matched_reader_add(&mut self, guid: GUID<PSM>);
    fn matched_reader_remove(&mut self, reader_proxy_guid: &GUID<PSM>);
    fn matched_reader_lookup(&self, a_reader_guid: GUID<PSM>) -> Option<&RTPSReaderProxy<PSM>>;
    fn is_acked_by_all(&self) -> bool;
}
