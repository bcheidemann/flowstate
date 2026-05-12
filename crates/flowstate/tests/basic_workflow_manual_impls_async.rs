use std::any::type_name;

use flowstate::{AsyncWorkflowState, prelude::*};

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

#[async_state]
trait BasicWorkflowState: Workflow {
    fn state_name(&self) -> String {
        self.state().name()
    }

    async fn next(self: Box<Self>) -> AsyncTransition<'static, WorkflowResult>;
}

#[async_state]
impl<State> AsyncWorkflowState<'static, WorkflowResult> for BasicWorkflow<State>
where
    State: flowstate::State + Send,
    BasicWorkflow<State>: BasicWorkflowState,
{
    fn name(&self) -> String {
        self.state_name()
    }

    async fn next(self: Box<Self>) -> AsyncTransition<'static, WorkflowResult> {
        BasicWorkflowState::next(self).await
    }
}

impl<State> BasicWorkflow<State> {
    fn transition<NewState>(self, next_state: NewState) -> AsyncTransition<'static, WorkflowResult>
    where
        BasicWorkflow<NewState>: AsyncWorkflowState<'static, WorkflowResult> + 'static,
    {
        AsyncTransition::Continue(Box::new(BasicWorkflow { _state: next_state }))
    }
}

struct StateA;

impl State for StateA {
    fn name(&self) -> String {
        type_name::<StateA>().to_string()
    }
}

#[async_state]
impl BasicWorkflowState for BasicWorkflow<StateA> {
    async fn next(self: Box<Self>) -> AsyncTransition<'static, WorkflowResult> {
        self.transition(StateB)
    }
}

struct StateB;

impl State for StateB {
    fn name(&self) -> String {
        type_name::<StateB>().to_string()
    }
}

#[async_state]
impl BasicWorkflowState for BasicWorkflow<StateB> {
    async fn next(self: Box<Self>) -> AsyncTransition<'static, WorkflowResult> {
        self.finish(WorkflowResult)
    }
}

#[derive(Debug, PartialEq)]
struct WorkflowResult;

#[tokio::test]
async fn test_basic_workflow_manual_impls_async() {
    let workflow = BasicWorkflow::init();
    let result = workflow.run().await;
    assert_eq!(result, WorkflowResult);
}
