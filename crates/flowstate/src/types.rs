use std::ops::ControlFlow;

use crate::WorkflowState;

/// Represents a transition to the next workflow state.
pub type Transition<'workflow, WorkflowResult> =
    ControlFlow<WorkflowResult, Box<dyn WorkflowState<'workflow, WorkflowResult> + 'workflow>>;

/// Shorthand for [`Transition<'static, WorkflowResult>`](Transition).
pub type StaticTransition<WorkflowResult> = Transition<'static, WorkflowResult>;
