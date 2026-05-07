use std::ops::ControlFlow;

use crate::WorkflowState;

pub type Transition<WorkflowResult> =
    ControlFlow<WorkflowResult, Box<dyn WorkflowState<WorkflowResult>>>;
