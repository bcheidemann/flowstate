mod utils;

use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Mutex},
};

use tracing::{Event, Metadata, Subscriber, span};

use crate::utils::HashMapFieldCollector;

#[derive(Default, Clone)]
pub struct TestSubscriber {
    inner: Arc<Mutex<TestSubscriberInner>>,
}

impl TestSubscriber {
    pub fn history(&self) -> SubscriberHistory {
        self.inner.lock().unwrap().history()
    }
}

impl Subscriber for TestSubscriber {
    fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, span: &span::Attributes<'_>) -> span::Id {
        let mut inner = self.inner.lock().unwrap();

        let span_id = inner.next_span_id();
        let parent_id = inner.current_span_id().cloned();

        inner.spans.push(FlatSpan {
            span_id: span_id.clone(),
            parent_id,
            follows_ids: Vec::new(),
            attributes: span.into(),
            events: Vec::new(),
        });

        span_id
    }

    fn record(&self, span: &span::Id, _values: &span::Record<'_>) {
        let mut inner = self.inner.lock().unwrap();

        let _span = inner.get_span_mut(span).expect("missing span");

        // TODO: Record values
    }

    fn record_follows_from(&self, span: &span::Id, follows: &span::Id) {
        let mut inner = self.inner.lock().unwrap();

        let span = inner.get_span_mut(span).expect("missing span");

        span.follows_ids.push(follows.clone());
    }

    fn event(&self, event: &Event<'_>) {
        let mut inner = self.inner.lock().unwrap();

        let event: RecordedEvent = event.into();

        match inner.current_span_mut() {
            Some(span) => span.events.push(event),
            None => inner.events.push(event),
        }
    }

    fn enter(&self, span: &span::Id) {
        self.inner.lock().unwrap().enter(span.clone());
    }

    fn exit(&self, span: &span::Id) {
        self.inner.lock().unwrap().exit(span);
    }
}

#[derive(Default)]
struct TestSubscriberInner {
    /// All spans.
    spans: Vec<FlatSpan>,
    /// Events which were emitted outside of any span.
    events: Vec<RecordedEvent>,
    /// The current stack of spans. First element corresponds to the outer-most
    /// span.
    stack: Vec<span::Id>,
}

impl TestSubscriberInner {
    fn next_span_id(&self) -> span::Id {
        span::Id::from_u64((self.spans.len() + 1) as u64)
    }

    fn get_span_mut(&mut self, span: &span::Id) -> Option<&mut FlatSpan> {
        let span_id = span.into_u64() as usize - 1;

        self.spans.get_mut(span_id)
    }

    fn current_span_id(&self) -> Option<&span::Id> {
        self.stack.last()
    }

    fn current_span_mut(&mut self) -> Option<&mut FlatSpan> {
        let current_span_id = self.current_span_id()?.clone();

        self.get_span_mut(&current_span_id)
    }

    fn enter(&mut self, span: span::Id) {
        self.stack.push(span);
    }

    fn exit(&mut self, span: &span::Id) {
        let span_to_exit = span;

        self.stack = self
            .stack
            .clone()
            .into_iter()
            .filter(|span_id| span_id.ne(span_to_exit))
            .collect();
    }

    fn history(&self) -> SubscriberHistory {
        let root_spans = self.spans.iter().filter(|span| span.parent_id.is_none());

        SubscriberHistory {
            spans: root_spans
                .map(|span| self.get_span_history(span.clone()))
                .collect(),
            root_events: self.events.clone(),
        }
    }

    fn get_span_history(&self, span: FlatSpan) -> HistorySpan {
        let children = self
            .spans
            .iter()
            .filter(|child_span| {
                let Some(parent_id) = &child_span.parent_id else {
                    return false;
                };
                *parent_id == span.span_id
            })
            .map(|span| self.get_span_history(span.clone()))
            .collect();

        HistorySpan {
            span,
            spans: children,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlatSpan {
    pub span_id: span::Id,
    pub parent_id: Option<span::Id>,
    pub follows_ids: Vec<span::Id>,
    pub attributes: SpanAttributes,
    pub events: Vec<RecordedEvent>,
}

#[derive(Debug, Clone)]
pub struct SpanAttributes {
    pub metadata: &'static Metadata<'static>,
    pub fields: HashMap<String, String>,
}

impl From<&span::Attributes<'_>> for SpanAttributes {
    fn from(value: &span::Attributes) -> Self {
        Self {
            metadata: value.metadata(),
            fields: HashMapFieldCollector::collect_span_fields(value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecordedEvent {
    pub metadata: &'static Metadata<'static>,
    pub fields: HashMap<String, String>,
}

impl From<&Event<'_>> for RecordedEvent {
    fn from(value: &Event<'_>) -> Self {
        Self {
            metadata: value.metadata(),
            fields: HashMapFieldCollector::collect_event_fields(value),
        }
    }
}

#[derive(Debug)]
pub struct SubscriberHistory {
    pub spans: Vec<HistorySpan>,
    pub root_events: Vec<RecordedEvent>,
}

#[derive(Debug)]
pub struct HistorySpan {
    pub span: FlatSpan,
    pub spans: Vec<HistorySpan>,
}

impl Deref for HistorySpan {
    type Target = FlatSpan;

    fn deref(&self) -> &Self::Target {
        &self.span
    }
}
