pub trait RtpsGroup {
    type Endpoints;

    fn endpoints(self) -> Self::Endpoints;
}
