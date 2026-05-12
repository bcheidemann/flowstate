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
    fn workflow_name(&self) -> String {
        type_name::<Self>().to_string()
    }

    fn state(&self) -> &dyn flowstate::State {
        &self._state
    }
}

trait BasicWorkflowState: Workflow {
    fn state_name(&self) -> String {
        self.state().name()
    }

    fn next(self: Box<Self>) -> Transition<'static, WorkflowResult>;
}

impl<State> WorkflowState<'static, WorkflowResult> for BasicWorkflow<State>
where
    State: flowstate::State,
    BasicWorkflow<State>: BasicWorkflowState,
{
    fn name(&self) -> String {
        self.state_name()
    }

    fn next(self: Box<Self>) -> Transition<'static, WorkflowResult> {
        BasicWorkflowState::next(self)
    }
}

impl<State> BasicWorkflow<State> {
    fn transition<NewState>(self, next_state: NewState) -> Transition<'static, WorkflowResult>
    where
        BasicWorkflow<NewState>: WorkflowState<'static, WorkflowResult> + 'static,
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

impl BasicWorkflowState for BasicWorkflow<StateA> {
    fn next(self: Box<Self>) -> Transition<'static, WorkflowResult> {
        self.transition(StateB)
    }
}

struct StateB;

impl State for StateB {
    fn name(&self) -> String {
        type_name::<StateB>().to_string()
    }
}

impl BasicWorkflowState for BasicWorkflow<StateB> {
    fn next(self: Box<Self>) -> Transition<'static, WorkflowResult> {
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
