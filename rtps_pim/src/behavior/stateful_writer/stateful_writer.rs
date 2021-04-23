use crate::{
    behavior::{self, RTPSWriter},
    structure::{self, types::GUID},
};

pub struct RTPSReaderProxy {

}

pub trait RTPSStatefulWriter<PSM: structure::Types + behavior::Types>: RTPSWriter<PSM> {
    fn matched_readers(&self) -> &[RTPSReaderProxy];
    fn matched_reader_add(&mut self, guid: GUID<PSM>);
    fn matched_reader_remove(&mut self, reader_proxy_guid: &GUID<PSM>);
    fn matched_reader_lookup(&self, a_reader_guid: GUID<PSM>) -> Option<&RTPSReaderProxy>;
    fn is_acked_by_all(&self) -> bool;
}
