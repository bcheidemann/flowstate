use flowstate::prelude::*;

#[derive(Workflow)]
#[flowstate(result = WorkflowResult)]
struct BasicWorkflow<State> {
    #[state]
    _state: State,
}

#[derive(State)]
struct StateA;

impl BasicWorkflowState for BasicWorkflow<StateA> {
    fn next(self: Box<Self>) -> Transition<WorkflowResult> {
        self.transition(StateB)
    }
}

#[derive(State)]
struct StateB;

impl BasicWorkflowState for BasicWorkflow<StateB> {
    fn next(self: Box<Self>) -> Transition<WorkflowResult> {
        self.finish(WorkflowResult)
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
