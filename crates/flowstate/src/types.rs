use std::ops::ControlFlow;

#[cfg(feature = "async")]
use crate::AsyncWorkflowState;
use crate::WorkflowState;

/// Represents a transition to the next workflow state.
pub type Transition<'workflow, WorkflowResult> =
    ControlFlow<WorkflowResult, Box<dyn WorkflowState<'workflow, WorkflowResult> + 'workflow>>;

/// Shorthand for [`Transition<'static, WorkflowResult>`](Transition).
pub type StaticTransition<WorkflowResult> = Transition<'static, WorkflowResult>;

/// Represents an async transition to the next workflow state.
#[cfg(feature = "async")]
pub type AsyncTransition<'workflow, WorkflowResult> =
    ControlFlow<WorkflowResult, Box<dyn AsyncWorkflowState<'workflow, WorkflowResult> + 'workflow>>;

/// Shorthand for [`AsyncTransition<'static, WorkflowResult>`](Transition).
#[cfg(feature = "async")]
pub type AsyncStaticTransition<WorkflowResult> = AsyncTransition<'static, WorkflowResult>;
