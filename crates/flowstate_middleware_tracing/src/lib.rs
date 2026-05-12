use tracing::{Instrument, Level, Span, debug_span, error_span, info_span, trace_span, warn_span};

use flowstate::middleware::{AsyncWorkflowMiddleware, WorkflowMetadata, WorkflowStateMetadata};

pub struct TracingMiddleware {
    pub workflow_span_level: Level,
    pub state_span_level: Level,
}

impl Default for TracingMiddleware {
    fn default() -> Self {
        Self {
            workflow_span_level: Level::TRACE,
            state_span_level: Level::TRACE,
        }
    }
}

impl AsyncWorkflowMiddleware for TracingMiddleware {
    fn wrap_workflow<'workflow, T: Send + 'workflow>(
        &self,
        metadata: &'workflow WorkflowMetadata<'workflow>,
        fut: impl Future<Output = T> + Send + 'workflow,
    ) -> impl Future<Output = T> + Send + 'workflow {
        fut.instrument(self.make_workflow_span(metadata))
    }

    fn wrap_state<'state, Transition: Send + 'state>(
        &self,
        metadata: &'state WorkflowStateMetadata<'state>,
        fut: impl Future<Output = Transition> + Send + 'state,
    ) -> impl Future<Output = Transition> + Send + 'state {
        fut.instrument(self.make_state_span(metadata))
    }
}

macro_rules! span {
    ($level:expr, $($rest:tt)*) => {
        match $level {
            Level::ERROR => error_span!($($rest)*),
            Level::WARN => warn_span!($($rest)*),
            Level::INFO => info_span!($($rest)*),
            Level::DEBUG => debug_span!($($rest)*),
            Level::TRACE => trace_span!($($rest)*),
        }
    };
}

impl TracingMiddleware {
    fn make_workflow_span(&self, metadata: &WorkflowMetadata<'_>) -> Span {
        span!(
            self.workflow_span_level,
            "workflow",
            workflow.name = metadata.name
        )
    }

    fn make_state_span(&self, metadata: &WorkflowStateMetadata<'_>) -> Span {
        span!(
            self.workflow_span_level,
            "state",
            state.name = metadata.name
        )
    }
}
