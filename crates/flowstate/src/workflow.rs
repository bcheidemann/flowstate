use std::ops::ControlFlow;

use crate::Transition;

/// A workflow that progresses through a series of [`WorkflowState`]s and
/// produces a [`Workflow::Result`].
///
/// Implement this trait on the workflow's container type. Call [`Workflow::run`]
/// to drive the workflow to completion and obtain the final result.
pub trait Workflow {
    type Result;

    fn run(self) -> Self::Result
    where
        Self: WorkflowState<Self::Result> + Sized,
    {
        let mut workflow: Box<dyn WorkflowState<Self::Result>> = Box::new(self);

        loop {
            match workflow.next() {
                ControlFlow::Continue(next) => workflow = next,
                ControlFlow::Break(result) => return result,
            }
        }
    }

    fn result(self, result: Self::Result) -> Transition<Self::Result>
    where
        Self: Sized,
    {
        ControlFlow::Break(result)
    }
}

/// A single state in a [`Workflow`].
///
/// Each implementation represents one state and defines the transition logic
/// in [`WorkflowState::next`]. Returning [`ControlFlow::Continue`] advances to
/// the next state, while [`ControlFlow::Break`] terminates the workflow with a
/// result.
pub trait WorkflowState<Result> {
    /// Consumes the current workflow state and returns either:
    ///
    /// 1. The next workflow state
    /// 2. The workflow result
    ///
    /// If using the [`Workflow`](flowstate_proc::Workflow) derive macro, return
    /// `self.transition(state)` to transition to the workflow to the next state.
    ///
    /// Return [`self.result(result)`](Workflow::result) to terminate the
    /// workflow with a result.
    fn next(self: Box<Self>) -> Transition<Result>;
}
