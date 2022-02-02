pub trait RtpsStructure {
    type StatelessWriter;
    type StatefulWriter;

    type StatelessReader;
    type StatefulReader;
}