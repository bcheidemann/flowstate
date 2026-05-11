use flowstate::prelude::*;

#[derive(Workflow)]
#[flowstate(
    is_async = true,
    result = WorkflowResult,
    state_trait = BasicWorkflowState,
)]
struct BasicWorkflow<State> {
    #[state]
    _state: State,
}

#[derive(State)]
struct StateA;

#[async_state]
impl BasicWorkflowState for BasicWorkflow<StateA> {
    async fn next(self: Box<Self>) -> AsyncStaticTransition<WorkflowResult> {
        self.transition(StateB)
    }
}

#[derive(State)]
struct StateB;

#[async_state]
impl BasicWorkflowState for BasicWorkflow<StateB> {
    async fn next(self: Box<Self>) -> AsyncStaticTransition<WorkflowResult> {
        self.finish(WorkflowResult)
    }
}

#[derive(Debug, PartialEq)]
struct WorkflowResult;

#[tokio::test]
async fn test_basic_workflow() {
    let workflow = BasicWorkflow::new(StateA);
    let result = workflow.run().await;
    assert_eq!(result, WorkflowResult);
}
