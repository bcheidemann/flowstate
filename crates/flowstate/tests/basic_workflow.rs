use std::ops::ControlFlow;

use flowstate::Workflow as _;

struct BasicWorkflow<State> {
    _state: State,
}

impl BasicWorkflow<StateA> {
    fn init() -> Self {
        Self { _state: StateA }
    }
}

impl<State> flowstate::Workflow for BasicWorkflow<State>
where
    BasicWorkflow<State>: flowstate::WorkflowState<WorkflowResult>,
{
    type Result = WorkflowResult;
}

impl<State> BasicWorkflow<State>
where
    BasicWorkflow<State>: flowstate::Workflow,
{
    fn transition<NewState: 'static>(
        self,
        next_state: NewState,
    ) -> ControlFlow<WorkflowResult, Box<dyn flowstate::WorkflowState<WorkflowResult>>>
    where
        BasicWorkflow<NewState>: flowstate::WorkflowState<WorkflowResult>,
    {
        ControlFlow::Continue(Box::new(BasicWorkflow { _state: next_state }))
    }
}

struct StateA;

impl flowstate::WorkflowState<WorkflowResult> for BasicWorkflow<StateA> {
    fn next(
        self: Box<Self>,
    ) -> ControlFlow<WorkflowResult, Box<dyn flowstate::WorkflowState<WorkflowResult>>> {
        self.transition(StateB)
    }
}

struct StateB;

impl flowstate::WorkflowState<WorkflowResult> for BasicWorkflow<StateB> {
    fn next(
        self: Box<Self>,
    ) -> ControlFlow<WorkflowResult, Box<dyn flowstate::WorkflowState<WorkflowResult>>> {
        self.result(WorkflowResult)
    }
}

#[derive(Debug, PartialEq)]
struct WorkflowResult;

#[test]
fn test() {
    let workflow = BasicWorkflow::init();
    let result = workflow.run();
    assert_eq!(result, WorkflowResult);
}
