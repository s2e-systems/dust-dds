pub trait RTPSGroup {
    type Endpoints;

    fn endpoints(self) -> Self::Endpoints;
}
