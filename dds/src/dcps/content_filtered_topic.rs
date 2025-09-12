pub struct ContentFilteredTopicEntity {
    name: String,
    related_topic_name: String,
    filter_expression: String,
    expression_parameters: Vec<String>,
}

impl ContentFilteredTopicEntity {
    pub fn new(
        name: String,
        related_topic_name: String,
        filter_expression: String,
        expression_parameters: Vec<String>,
    ) -> Self {
        Self {
            name,
            related_topic_name,
            filter_expression,
            expression_parameters,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn related_topic_name(&self) -> &str {
        &self.related_topic_name
    }

    pub fn filter_expression(&self) -> &str {
        &self.filter_expression
    }

    pub fn expression_parameters(&self) -> &[String] {
        &self.expression_parameters
    }

    pub fn set_expression_parameters(&mut self, expression_parameters: Vec<String>) {
        self.expression_parameters = expression_parameters;
    }
}
