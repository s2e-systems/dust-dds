use rtps_pim::{
    messages::{
        overall_structure::{RtpsMessage, RtpsSubmessageType},
        submessage_elements::Parameter,
        types::FragmentNumber,
    },
    structure::types::{Locator, SequenceNumber},
};

pub trait TransportWrite {
    fn write(
        &mut self,
        message: &RtpsMessage<
            Vec<
                RtpsSubmessageType<
                    Vec<SequenceNumber>,
                    Vec<Parameter<'_>>,
                    &'_ [u8],
                    Vec<Locator>,
                    Vec<FragmentNumber>,
                >,
            >,
        >,
        destination_locator: Locator,
    );
}

pub trait TransportRead {
    fn read(
        &mut self,
    ) -> Option<(
        Locator,
        RtpsMessage<
            Vec<
                RtpsSubmessageType<
                    Vec<SequenceNumber>,
                    Vec<Parameter<'_>>,
                    &'_ [u8],
                    Vec<Locator>,
                    Vec<FragmentNumber>,
                >,
            >,
        >,
    )>;
}
