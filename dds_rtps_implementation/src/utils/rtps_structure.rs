pub trait RtpsStructure {
    type Participant;

    type StatelessWriter;
    type StatefulWriter;

    type StatelessReader;
    type StatefulReader;
}