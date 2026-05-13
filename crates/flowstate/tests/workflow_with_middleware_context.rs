use flowstate::{Context, TypedKey, WorkflowState, prelude::*};

#[derive(Workflow)]
#[flowstate(
    result = WorkflowResult,
    state_trait = BasicWorkflowState,
    ctx.a.b.c.0 = "first (workflow)",
    ctx.a.b.c.1 = "second (workflow)",
)]
struct BasicWorkflow<State> {
    #[state]
    _state: State,
}

#[derive(State)]
#[flowstate(
    ctx.c.b.a.0 = "first (state)",
    ctx.c.b.a.1 = "second (state)",
)]
struct StateA;

impl BasicWorkflowState for BasicWorkflow<StateA> {
    fn next(self: Box<Self>) -> StaticTransition<WorkflowResult> {
        self.finish(WorkflowResult)
    }
}

struct WorkflowResult;

#[test]
fn workflow_with_middleware_context() {
    let mut workflow_context = Context::default();
    let mut state_context = Context::default();
    let workflow = BasicWorkflow::new(StateA);

    Workflow::record_workflow_context(&workflow, &mut workflow_context);
    WorkflowState::record_context(&workflow, &mut state_context);

    let workflow_first = workflow_context.get::<&'static str>(&TypedKey::new("a.b.c.0"));
    let workflow_second = workflow_context.get::<&'static str>(&TypedKey::new("a.b.c.1"));
    let workflow_missing = workflow_context.get::<&'static str>(&TypedKey::new("missing.key"));

    dbg!(workflow_context.keys().collect::<Vec<_>>());

    assert_eq!(workflow_first, Some(&"first (workflow)"));
    assert_eq!(workflow_second, Some(&"second (workflow)"));
    assert_eq!(workflow_missing, None);

    let state_first = state_context.get::<&'static str>(&TypedKey::new("c.b.a.0"));
    let state_second = state_context.get::<&'static str>(&TypedKey::new("c.b.a.1"));
    let state_missing = state_context.get::<&'static str>(&TypedKey::new("missing.key"));

    dbg!(state_context.keys().collect::<Vec<_>>());
    assert_eq!(state_first, Some(&"first (state)"));
    assert_eq!(state_second, Some(&"second (state)"));
    assert_eq!(state_missing, None);
}
