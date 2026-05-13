#[cfg(feature = "async")]
use tracing::Instrument;
use tracing::{Span, trace_span};

#[cfg(feature = "async")]
use flowstate::{AsyncContext, middleware::AsyncWorkflowMiddleware};
use flowstate::{
    Context, TypedKey,
    middleware::{WorkflowMetadata, WorkflowMiddleware, WorkflowStateMetadata},
};

pub struct TracingMiddleware<
    W = fn(&WorkflowMetadata) -> Option<Span>,
    S = fn(&WorkflowStateMetadata) -> Option<Span>,
> {
    workflow_span_factory: W,
    state_span_factory: S,
}

impl Default for TracingMiddleware {
    fn default() -> Self {
        Self {
            workflow_span_factory: |m| {
                Some(trace_span!("flowstate::Workflow", workflow.name = m.name))
            },
            state_span_factory: |m| {
                Some(trace_span!("flowstate::WorkflowState", state.name = m.name))
            },
        }
    }
}

impl<W, S> WorkflowMiddleware for TracingMiddleware<W, S>
where
    W: WorkflowSpanFactory,
    S: StateSpanFactory,
{
    fn wrap_workflow<'workflow, Result>(
        &self,
        metadata: &'workflow WorkflowMetadata<'workflow>,
        ctx: &Context,
        next: impl FnOnce() -> Result,
    ) -> impl FnOnce() -> Result {
        || {
            if let Some(span) = ctx.get::<Span>(&TypedKey::new("span")) {
                let _guard = span.enter();
                return next();
            }
            if let Some(span) = self.workflow_span_factory.make_workflow_span(metadata) {
                let _guard = span.enter();
                return next();
            }
            next()
        }
    }

    fn wrap_state<'state, Transition>(
        &self,
        metadata: &'state WorkflowStateMetadata<'state>,
        ctx: &Context,
        next: impl FnOnce() -> Transition,
    ) -> impl FnOnce() -> Transition {
        || {
            if let Some(span) = ctx.get::<Span>(&TypedKey::new("span")) {
                let _guard = span.enter();
                return next();
            }
            if let Some(span) = self.state_span_factory.make_state_span(metadata) {
                let _guard = span.enter();
                return next();
            }
            next()
        }
    }
}

#[cfg(feature = "async")]
impl<W, S> AsyncWorkflowMiddleware for TracingMiddleware<W, S>
where
    W: WorkflowSpanFactory,
    S: StateSpanFactory,
{
    fn wrap_workflow<'workflow, T: Send + 'workflow>(
        &self,
        metadata: &'workflow WorkflowMetadata<'workflow>,
        ctx: &AsyncContext,
        fut: impl Future<Output = T> + Send + 'workflow,
    ) -> impl Future<Output = T> + Send + 'workflow {
        if let Some(span) = ctx.get::<Span>(&TypedKey::new("span")) {
            return fut.instrument(span.clone());
        }
        if let Some(span) = self.workflow_span_factory.make_workflow_span(metadata) {
            return fut.instrument(span.clone());
        }
        fut.instrument(Span::none())
    }

    fn wrap_state<'state, Transition: Send + 'state>(
        &self,
        metadata: &'state WorkflowStateMetadata<'state>,
        ctx: &AsyncContext,
        fut: impl Future<Output = Transition> + Send + 'state,
    ) -> impl Future<Output = Transition> + Send + 'state {
        if let Some(span) = ctx.get::<Span>(&TypedKey::new("span")) {
            return fut.instrument(span.clone());
        }
        if let Some(span) = self.state_span_factory.make_state_span(metadata) {
            return fut.instrument(span.clone());
        }
        fut.instrument(Span::none())
    }
}

impl<W, S> TracingMiddleware<W, S> {
    pub fn with_workflow_span<W2: WorkflowSpanFactory>(
        self,
        workflow_span_factory: W2,
    ) -> TracingMiddleware<W2, S> {
        TracingMiddleware {
            workflow_span_factory,
            state_span_factory: self.state_span_factory,
        }
    }

    pub fn with_state_span<S2: StateSpanFactory>(
        self,
        state_span_factory: S2,
    ) -> TracingMiddleware<W, S2> {
        TracingMiddleware {
            workflow_span_factory: self.workflow_span_factory,
            state_span_factory,
        }
    }
}

pub trait WorkflowSpanFactory {
    fn make_workflow_span(&self, metadata: &WorkflowMetadata) -> Option<Span>;
}

impl<F: Fn(&WorkflowMetadata) -> Option<Span>> WorkflowSpanFactory for F {
    fn make_workflow_span(&self, metadata: &WorkflowMetadata) -> Option<Span> {
        self(metadata)
    }
}

impl WorkflowSpanFactory for () {
    fn make_workflow_span(&self, _metadata: &WorkflowMetadata) -> Option<Span> {
        None
    }
}

pub trait StateSpanFactory {
    fn make_state_span(&self, metadata: &WorkflowStateMetadata) -> Option<Span>;
}

impl<F: Fn(&WorkflowStateMetadata) -> Option<Span>> StateSpanFactory for F {
    fn make_state_span(&self, metadata: &WorkflowStateMetadata) -> Option<Span> {
        self(metadata)
    }
}

impl StateSpanFactory for () {
    fn make_state_span(&self, _metadata: &WorkflowStateMetadata) -> Option<Span> {
        None
    }
}
