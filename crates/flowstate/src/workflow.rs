use std::ops::ControlFlow;

use crate::{State, Transition};

pub trait Workflow {
    fn state(&self) -> &dyn State;
}

/// A [`Workflow`] in a specific state.
///
/// Each implementation represents one state and defines the transition logic
/// in [`WorkflowState::next`]. Returning [`ControlFlow::Continue`] advances to
/// the next state, while [`ControlFlow::Break`] terminates the workflow with a
/// result.
pub trait WorkflowState<Result>: Workflow {
    /// Consumes the current workflow state and returns either:
    ///
    /// 1. The next workflow state
    /// 2. The workflow result
    ///
    /// Return [`self.finish(result)`](Workflow::finish) or
    /// [`self.finish_with(|workflow| result)`](Workflow::finish_with) to
    /// terminate the workflow with a result.
    fn next(self: Box<Self>) -> Transition<Result>;

    fn run(self) -> Result
    where
        Self: WorkflowState<Result> + Sized,
    {
        let mut workflow: Box<dyn WorkflowState<Result>> = Box::new(self);

        loop {
            match workflow.next() {
                ControlFlow::Continue(next) => workflow = next,
                ControlFlow::Break(result) => return result,
            }
        }
    }

    fn finish(self, result: Result) -> Transition<Result>
    where
        Self: Sized,
    {
        ControlFlow::Break(result)
    }

    fn finish_with<Fn>(self, map_fn: Fn) -> Transition<Result>
    where
        Self: Sized,
        Fn: FnOnce(Self) -> Result,
    {
        ControlFlow::Break(map_fn(self))
    }
}
