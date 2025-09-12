use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct TopicDescription(
    dust_dds::topic_definition::topic_description::TopicDescription<
        dust_dds::std_runtime::StdRuntime,
    >,
);

impl
    From<
        dust_dds::topic_definition::topic_description::TopicDescription<
            dust_dds::std_runtime::StdRuntime,
        >,
    > for TopicDescription
{
    fn from(
        value: dust_dds::topic_definition::topic_description::TopicDescription<
            dust_dds::std_runtime::StdRuntime,
        >,
    ) -> Self {
        Self(value)
    }
}

impl
    AsRef<
        dust_dds::topic_definition::topic_description::TopicDescription<
            dust_dds::std_runtime::StdRuntime,
        >,
    > for TopicDescription
{
    fn as_ref(
        &self,
    ) -> &dust_dds::topic_definition::topic_description::TopicDescription<
        dust_dds::std_runtime::StdRuntime,
    > {
        &self.0
    }
}
