pub trait SubmessageProducer<'a> {
    type DestinedSubmessageType;

    fn produce_submessages(&'a mut self) -> Self::DestinedSubmessageType;
}
