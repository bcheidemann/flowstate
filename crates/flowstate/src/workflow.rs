use std::ops::ControlFlow;

use async_trait::async_trait;

use crate::{AsyncTransition, State, Transition};

pub trait Workflow {
    fn state(&self) -> &dyn State;
}

/// A [`Workflow`] in a specific state.
///
/// Each implementation represents one state and defines the transition logic
/// in [`WorkflowState::next`]. Returning [`ControlFlow::Continue`] advances to
/// the next state, while [`ControlFlow::Break`] terminates the workflow with a
/// result.
pub trait WorkflowState<'workflow, Result>: Workflow {
    fn name(&self) -> String {
        self.state().name()
    }

    /// Consumes the current workflow state and returns either:
    ///
    /// 1. The next workflow state
    /// 2. The workflow result
    ///
    /// Return [`self.finish(result)`](Workflow::finish) or
    /// [`self.finish_with(|workflow| result)`](Workflow::finish_with) to
    /// terminate the workflow with a result.
    fn next(self: Box<Self>) -> Transition<'workflow, Result>;

    fn run(self) -> Result
    where
        Self: WorkflowState<'workflow, Result> + Sized,
    {
        let mut workflow: Box<dyn WorkflowState<Result>> = Box::new(self);

        loop {
            match workflow.next() {
                ControlFlow::Continue(next) => workflow = next,
                ControlFlow::Break(result) => return result,
            }
        }
    }

    fn finish(self, result: Result) -> Transition<'workflow, Result>
    where
        Self: Sized,
    {
        ControlFlow::Break(result)
    }

    fn finish_with<Fn>(self, map_fn: Fn) -> Transition<'workflow, Result>
    where
        Self: Sized,
        Fn: FnOnce(Self) -> Result,
    {
        ControlFlow::Break(map_fn(self))
    }
}

/// An async [`Workflow`] in a specific state.
///
/// Each implementation represents one state and defines the transition logic
/// in [`WorkflowState::next`]. Returning [`ControlFlow::Continue`] advances to
/// the next state, while [`ControlFlow::Break`] terminates the workflow with a
/// result.
#[cfg(feature = "async")]
#[async_trait]
pub trait AsyncWorkflowState<'workflow, Result>: Workflow + Send {
    fn name(&self) -> String {
        self.state().name()
    }

    /// Consumes the current workflow state and returns either:
    ///
    /// 1. The next workflow state
    /// 2. The workflow result
    ///
    /// Return [`self.finish(result)`](Workflow::finish) or
    /// [`self.finish_with(|workflow| result)`](Workflow::finish_with) to
    /// terminate the workflow with a result.
    async fn next(self: Box<Self>) -> AsyncTransition<'workflow, Result>;

    async fn run(self) -> Result
    where
        Self: AsyncWorkflowState<'workflow, Result> + Sized,
    {
        let mut workflow: Box<dyn AsyncWorkflowState<Result>> = Box::new(self);

        loop {
            match workflow.next().await {
                ControlFlow::Continue(next) => workflow = next,
                ControlFlow::Break(result) => return result,
            }
        }
    }

    fn finish(self, result: Result) -> AsyncTransition<'workflow, Result>
    where
        Self: Sized,
    {
        ControlFlow::Break(result)
    }

    fn finish_with<Fn>(self, map_fn: Fn) -> AsyncTransition<'workflow, Result>
    where
        Self: Sized,
        Fn: FnOnce(Self) -> Result,
    {
        ControlFlow::Break(map_fn(self))
    }
}
