use flowstate::prelude::*;
use flowstate_middleware_tracing::TracingMiddleware;
use tracing::instrument::WithSubscriber;
use tracing_test_subscriber::TestSubscriber;

#[derive(Workflow)]
#[flowstate(
    is_async = true,
    result = WorkflowResult,
    state_trait = BasicWorkflowState,
    name = "BasicWorkflow"
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
async fn test_tracing_middleware_async() {
    let subscriber = TestSubscriber::default();
    let workflow = BasicWorkflow::new(StateA);
    let middleware = TracingMiddleware::default();

    let result = workflow
        .run_with_middleware(middleware)
        .with_subscriber(subscriber.clone())
        .await;

    let history = subscriber.history();

    assert_eq!(history.spans.len(), 1);
    assert_eq!(
        history.spans[0].attributes.fields.get("workflow.name"),
        Some(&"\"BasicWorkflow\"".to_string())
    );
    assert_eq!(history.spans[0].spans.len(), 2);
    assert_eq!(
        history.spans[0].spans[0]
            .attributes
            .fields
            .get("state.name"),
        Some(&"\"tracing_middleware_async::StateA\"".to_string())
    );
    assert_eq!(
        history.spans[0].spans[1]
            .attributes
            .fields
            .get("state.name"),
        Some(&"\"tracing_middleware_async::StateB\"".to_string())
    );
    assert!(history.root_events.is_empty());

    assert_eq!(result, WorkflowResult);
}
