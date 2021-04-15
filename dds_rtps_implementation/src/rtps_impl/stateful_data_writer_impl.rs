use rust_dds_api::return_type::DDSResult;


pub struct StatefulDataWriterImpl {
}

impl StatefulDataWriterImpl {
    pub fn new() -> Self {
        Self {  }
    }

    pub fn write_w_timestamp(&mut self) -> DDSResult<()> {
        todo!()
        // let kind = ChangeKind::Alive;
        // let data = vec![0, 1, 2];
        // let inline_qos = vec![];
        // let handle = 1;
        // let change = self.new_change(kind, data, inline_qos, handle);
        // self.writer_cache.add_change(change);
        // Ok(())
    }
}