# `flowstate_middleware_tracing`

Tracing middleware for flowstate.

## Usage

```rs
use flowstate::prelude::*;

#[derive(Workflow)]
#[flowstate(
    result = MyWorkflowResult,
    state_trait = MyWorkflowState,
    ctx.span = info_span!(
        "MyWorkflow",
        description = "A flowstate workflow",
        ctx = self.ctx,
    ),
)]
struct MyWorkflow<State> {
    #[state]
    _state: State,
    ctx: String,
}

#[derive(State)]
#[flowstate(
    ctx.span = info_span!(
        "StateA",
        data = self.data,
    ),
)]
struct StateA {
    data: &'static str,
}

impl BasicWorkflowState for BasicWorkflow<StateB> {
    fn next(self: Box<Self>) -> StaticTransition<WorkflowResult> {
        info!("event B");
        self.finish(WorkflowResult)
    }
}

// ...

fn main() {
    let _ = MyWorkflow::new(StateA).run_with_middleware(TracingMiddleware::default());
}
```
