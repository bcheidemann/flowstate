use std::any::type_name;

use flowstate::{WorkflowState, prelude::*};

struct BasicWorkflow<State> {
    _state: State,
}

impl BasicWorkflow<StateA> {
    fn init() -> Self {
        Self { _state: StateA }
    }
}

impl<State: flowstate::State> Workflow for BasicWorkflow<State> {
    fn state(&self) -> &dyn flowstate::State {
        &self._state
    }
}

impl<State> BasicWorkflow<State> {
    fn transition<NewState: 'static>(self, next_state: NewState) -> Transition<WorkflowResult>
    where
        BasicWorkflow<NewState>: WorkflowState<WorkflowResult>,
    {
        Transition::Continue(Box::new(BasicWorkflow { _state: next_state }))
    }
}

struct StateA;

impl State for StateA {
    fn name(&self) -> String {
        type_name::<StateA>().to_string()
    }
}

impl WorkflowState<WorkflowResult> for BasicWorkflow<StateA> {
    fn next(self: Box<Self>) -> Transition<WorkflowResult> {
        self.transition(StateB)
    }
}

struct StateB;

impl State for StateB {
    fn name(&self) -> String {
        type_name::<StateB>().to_string()
    }
}

impl WorkflowState<WorkflowResult> for BasicWorkflow<StateB> {
    fn next(self: Box<Self>) -> Transition<WorkflowResult> {
        self.finish(WorkflowResult)
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
