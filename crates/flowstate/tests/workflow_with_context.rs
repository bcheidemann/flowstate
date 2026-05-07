use flowstate::{Workflow, WorkflowState};

struct WorkflowContext {
    result: String,
}

#[derive(flowstate::Workflow)]
#[flowstate(result = String)]
struct WorkflowWithContext<State> {
    #[state]
    _state: State,
    ctx: WorkflowContext,
}

struct StateA;

impl WorkflowState<String> for WorkflowWithContext<StateA> {
    fn next(self: Box<Self>) -> flowstate::Transition<String> {
        self.transition(StateB)
    }
}

struct StateB;

impl WorkflowState<String> for WorkflowWithContext<StateB> {
    fn next(self: Box<Self>) -> flowstate::Transition<String> {
        let result = self.ctx.result.clone();

        self.result(result)
    }
}

#[test]
fn test_workflow_with_context() {
    let workflow = WorkflowWithContext::new(
        StateA,
        WorkflowContext {
            result: "Hello World!".to_string(),
        },
    );

    let result = workflow.run();

    assert_eq!(result, "Hello World!");
}
