use std::collections::HashMap;

use tracing::{field::Visit, span};

#[derive(Default)]
pub struct HashMapFieldCollector(HashMap<String, String>);

impl Visit for HashMapFieldCollector {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn core::fmt::Debug) {
        self.0.insert(field.to_string(), format!("{value:?}"));
    }
}

impl HashMapFieldCollector {
    pub fn collect_span(span: &span::Attributes<'_>) -> HashMap<String, String> {
        let mut collector = Self::default();
        span.record(&mut collector);
        collector.into_fields()
    }

    pub fn into_fields(self) -> HashMap<String, String> {
        self.0
    }
}
