use std::ops::ControlFlow;

use flowstate::{Transition, Workflow, WorkflowState};

struct BasicWorkflow<State> {
    _state: State,
}

impl BasicWorkflow<StateA> {
    fn init() -> Self {
        Self { _state: StateA }
    }
}

impl<State> Workflow for BasicWorkflow<State>
where
    BasicWorkflow<State>: WorkflowState<WorkflowResult>,
{
    type Result = WorkflowResult;
}

impl<State> BasicWorkflow<State>
where
    BasicWorkflow<State>: Workflow,
{
    fn transition<NewState: 'static>(self, next_state: NewState) -> Transition<WorkflowResult>
    where
        BasicWorkflow<NewState>: WorkflowState<WorkflowResult>,
    {
        ControlFlow::Continue(Box::new(BasicWorkflow { _state: next_state }))
    }
}

struct StateA;

impl WorkflowState<WorkflowResult> for BasicWorkflow<StateA> {
    fn next(self: Box<Self>) -> Transition<WorkflowResult> {
        self.transition(StateB)
    }
}

struct StateB;

impl WorkflowState<WorkflowResult> for BasicWorkflow<StateB> {
    fn next(self: Box<Self>) -> Transition<WorkflowResult> {
        self.result(WorkflowResult)
    }
}

#[derive(Debug, PartialEq)]
struct WorkflowResult;

#[test]
fn test_basic_workflow_manual_impls() {
    let workflow = BasicWorkflow::init();
    let result = workflow.run();
    assert_eq!(result, WorkflowResult);
}
