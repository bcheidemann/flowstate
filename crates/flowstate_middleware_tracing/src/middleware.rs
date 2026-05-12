use tracing::{Instrument, Span, trace_span};

use flowstate::middleware::{
    AsyncWorkflowMiddleware, WorkflowMetadata, WorkflowMiddleware, WorkflowStateMetadata,
};

pub struct TracingMiddleware<
    W = fn(&WorkflowMetadata) -> Span,
    S = fn(&WorkflowStateMetadata) -> Span,
> {
    workflow_span_factory: W,
    state_span_factory: S,
}

impl Default for TracingMiddleware {
    fn default() -> Self {
        Self {
            workflow_span_factory: |m| trace_span!("flowstate::Workflow", workflow.name = m.name),
            state_span_factory: |m| trace_span!("flowstate::WorkflowState", state.name = m.name),
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
        next: impl FnOnce() -> Result,
    ) -> impl FnOnce() -> Result {
        || {
            let span = self.workflow_span_factory.make_workflow_span(metadata);
            let _guard = span.enter();
            next()
        }
    }

    fn wrap_state<'state, Transition>(
        &self,
        metadata: &'state WorkflowStateMetadata<'state>,
        next: impl FnOnce() -> Transition,
    ) -> impl FnOnce() -> Transition {
        || {
            let span = self.state_span_factory.make_state_span(metadata);
            let _guard = span.enter();
            next()
        }
    }
}

impl<W, S> AsyncWorkflowMiddleware for TracingMiddleware<W, S>
where
    W: WorkflowSpanFactory,
    S: StateSpanFactory,
{
    fn wrap_workflow<'workflow, T: Send + 'workflow>(
        &self,
        metadata: &'workflow WorkflowMetadata<'workflow>,
        fut: impl Future<Output = T> + Send + 'workflow,
    ) -> impl Future<Output = T> + Send + 'workflow {
        fut.instrument(self.workflow_span_factory.make_workflow_span(metadata))
    }

    fn wrap_state<'state, Transition: Send + 'state>(
        &self,
        metadata: &'state WorkflowStateMetadata<'state>,
        fut: impl Future<Output = Transition> + Send + 'state,
    ) -> impl Future<Output = Transition> + Send + 'state {
        fut.instrument(self.state_span_factory.make_state_span(metadata))
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
    fn make_workflow_span(&self, metadata: &WorkflowMetadata) -> Span;
}

impl<F: Fn(&WorkflowMetadata) -> Span> WorkflowSpanFactory for F {
    fn make_workflow_span(&self, metadata: &WorkflowMetadata) -> Span {
        self(metadata)
    }
}

pub trait StateSpanFactory {
    fn make_state_span(&self, metadata: &WorkflowStateMetadata) -> Span;
}

impl<F: Fn(&WorkflowStateMetadata) -> Span> StateSpanFactory for F {
    fn make_state_span(&self, metadata: &WorkflowStateMetadata) -> Span {
        self(metadata)
    }
}
