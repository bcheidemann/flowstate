use flowstate::{
    middleware::{WorkflowMetadata, WorkflowStateMetadata},
    prelude::*,
};
use flowstate_middleware_tracing::TracingMiddleware;
use tracing::{Level, info, info_span, instrument::WithSubscriber};
use tracing_test_subscriber::TestSubscriber;

#[derive(AsyncWorkflow)]
#[flowstate(
    result = WorkflowResult,
    state_trait = BasicWorkflowState,
    name = "BasicWorkflow"
)]
struct BasicWorkflow<State> {
    #[state]
    _state: State,
}

#[derive(AsyncState)]
struct StateA;

#[async_state]
impl BasicWorkflowState for BasicWorkflow<StateA> {
    async fn next(self: Box<Self>) -> AsyncStaticTransition<WorkflowResult> {
        info!("event A");
        self.transition(StateB {
            data: "Hello world!",
        })
    }
}

#[derive(AsyncState)]
#[flowstate(
    ctx.span = info_span!(
        "StateB",
        data = self.data,
    )
)]
struct StateB {
    data: &'static str,
}

#[async_state]
impl BasicWorkflowState for BasicWorkflow<StateB> {
    async fn next(self: Box<Self>) -> AsyncStaticTransition<WorkflowResult> {
        info!("event B");
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

    // Outer span
    assert_eq!(history.spans.len(), 1);
    assert_eq!(history.spans[0].attributes.metadata.level(), &Level::TRACE);
    assert_eq!(
        history.spans[0].attributes.metadata.name(),
        "flowstate::Workflow"
    );
    assert_eq!(
        history.spans[0].attributes.fields.get("workflow.name"),
        Some(&"\"BasicWorkflow\"".to_string())
    );
    assert_eq!(history.spans[0].spans.len(), 2);

    // State A
    assert_eq!(
        history.spans[0].spans[0].attributes.metadata.level(),
        &Level::TRACE
    );
    assert_eq!(
        history.spans[0].spans[0].attributes.metadata.name(),
        "flowstate::WorkflowState"
    );
    assert_eq!(
        history.spans[0].spans[0]
            .attributes
            .fields
            .get("state.name"),
        Some(&"\"tracing_middleware_async::StateA\"".to_string())
    );
    assert_eq!(history.spans[0].spans[0].events.len(), 1);
    assert_eq!(
        history.spans[0].spans[0].events[0].fields.get("message"),
        Some(&"event A".to_string())
    );

    // State B
    assert_eq!(
        history.spans[0].spans[1].attributes.metadata.level(),
        &Level::INFO
    );
    assert_eq!(
        history.spans[0].spans[1].attributes.metadata.name(),
        "StateB"
    );
    assert_eq!(
        history.spans[0].spans[1].attributes.fields.get("data"),
        Some(&"\"Hello world!\"".to_string())
    );
    assert_eq!(history.spans[0].spans[0].events.len(), 1);
    assert_eq!(
        history.spans[0].spans[1].events[0].fields.get("message"),
        Some(&"event B".to_string())
    );

    // Root events
    assert!(history.root_events.is_empty());

    // Result
    assert_eq!(result, WorkflowResult);
}

#[tokio::test]
async fn test_tracing_middleware_custom_spans_async() {
    let subscriber = TestSubscriber::default();
    let workflow = BasicWorkflow::new(StateA);
    let middleware = TracingMiddleware::default()
        .with_workflow_span(|m: &WorkflowMetadata| {
            Some(info_span!(
                "custom workflow span",
                my_workflow_name = m.name
            ))
        })
        .with_state_span(|m: &WorkflowStateMetadata| {
            Some(info_span!("custom state span", my_state_name = m.name))
        });

    let result = workflow
        .run_with_middleware(middleware)
        .with_subscriber(subscriber.clone())
        .await;

    let history = subscriber.history();

    // Outer span
    assert_eq!(history.spans.len(), 1);
    assert_eq!(history.spans[0].attributes.metadata.level(), &Level::INFO);
    assert_eq!(
        history.spans[0].attributes.metadata.name(),
        "custom workflow span"
    );
    assert_eq!(
        history.spans[0].attributes.fields.get("my_workflow_name"),
        Some(&"\"BasicWorkflow\"".to_string())
    );
    assert_eq!(history.spans[0].spans.len(), 2);

    // State A
    assert_eq!(
        history.spans[0].spans[0].attributes.metadata.level(),
        &Level::INFO
    );
    assert_eq!(
        history.spans[0].spans[0].attributes.metadata.name(),
        "custom state span"
    );
    assert_eq!(
        history.spans[0].spans[0]
            .attributes
            .fields
            .get("my_state_name"),
        Some(&"\"tracing_middleware_async::StateA\"".to_string())
    );
    assert_eq!(history.spans[0].spans[0].events.len(), 1);
    assert_eq!(
        history.spans[0].spans[0].events[0].fields.get("message"),
        Some(&"event A".to_string())
    );

    // State B
    assert_eq!(
        history.spans[0].spans[1].attributes.metadata.level(),
        &Level::INFO
    );
    assert_eq!(
        history.spans[0].spans[1].attributes.metadata.name(),
        "StateB"
    );
    assert_eq!(
        history.spans[0].spans[1].attributes.fields.get("data"),
        Some(&"\"Hello world!\"".to_string())
    );
    assert_eq!(history.spans[0].spans[1].events.len(), 1);
    assert_eq!(
        history.spans[0].spans[1].events[0].fields.get("message"),
        Some(&"event B".to_string())
    );

    // Root events
    assert!(history.root_events.is_empty());

    // Result
    assert_eq!(result, WorkflowResult);
}
