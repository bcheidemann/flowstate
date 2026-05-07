use flowstate::{Transition, Workflow as _, WorkflowState};

#[derive(flowstate::Workflow)]
#[flowstate(result = WorkflowResult)]
struct BasicWorkflow<State> {
    #[state]
    _state: State,
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
fn test_basic_workflow() {
    let workflow = BasicWorkflow::new(StateA);
    let result = workflow.run();
    assert_eq!(result, WorkflowResult);
}
