pub trait RtpsStructure {
    type Group;
    type Participant;

    type StatelessWriter;
    type StatefulWriter;

    type StatelessReader;
    type StatefulReader;
}