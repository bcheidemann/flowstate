use flowstate::prelude::*;

struct WorkflowContext {
    result: String,
}

#[derive(Workflow)]
#[flowstate(result = String)]
struct WorkflowWithContext<State> {
    #[state]
    _state: State,
    ctx: WorkflowContext,
}

#[derive(State)]
struct StateA;

impl WorkflowWithContextState for WorkflowWithContext<StateA> {
    fn next(self: Box<Self>) -> flowstate::Transition<String> {
        self.transition(StateB)
    }
}

#[derive(State)]
struct StateB;

impl WorkflowWithContextState for WorkflowWithContext<StateB> {
    fn next(self: Box<Self>) -> flowstate::Transition<String> {
        let result = self.ctx.result.clone();

        self.finish(result)
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
