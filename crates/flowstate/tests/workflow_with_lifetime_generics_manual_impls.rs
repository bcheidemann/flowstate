use std::any::type_name;

use flowstate::{WorkflowState, prelude::*};

struct StrContainer<'a> {
    #[allow(unused)]
    my_str: &'a str,
}

struct MyWorkflow<'a, State> {
    state: State,
    my_str: &'a str,
    my_str_container: StrContainer<'a>,
    message: String,
}

impl<'a, State> MyWorkflow<'a, State> {
    fn new(
        state: State,
        my_str: &'a str,
        my_str_container: StrContainer<'a>,
        message: String,
    ) -> Self {
        Self {
            state,
            my_str,
            my_str_container,
            message,
        }
    }
}

impl<'a, State: flowstate::State> Workflow for MyWorkflow<'a, State> {
    fn state(&self) -> &dyn flowstate::State {
        &self.state
    }
}

trait MyWorkflowState<'workflow>: Workflow {
    fn state_name(&self) -> String {
        self.state().name()
    }

    fn next(self: Box<Self>) -> Transition<'workflow, WorkflowResult>;
}

impl<'workflow, State> WorkflowState<'workflow, WorkflowResult> for MyWorkflow<'workflow, State>
where
    State: flowstate::State,
    MyWorkflow<'workflow, State>: MyWorkflowState<'workflow>,
{
    fn name(&self) -> String {
        self.state_name()
    }

    fn next(self: Box<Self>) -> Transition<'workflow, WorkflowResult> {
        MyWorkflowState::next(self)
    }
}

impl<'workflow, State> MyWorkflow<'workflow, State> {
    fn transition<R, NewState>(self, new_state: NewState) -> Transition<'workflow, R>
    where
        MyWorkflow<'workflow, NewState>: WorkflowState<'workflow, R> + 'workflow,
    {
        Transition::Continue(Box::new(MyWorkflow {
            state: new_state,
            my_str: self.my_str,
            my_str_container: self.my_str_container,
            message: self.message,
        }))
    }
}

struct StateA<'a>(&'a str);

impl State for StateA<'_> {
    fn name(&self) -> String {
        type_name::<StateA>().to_string()
    }
}

impl<'workflow> MyWorkflowState<'workflow> for MyWorkflow<'workflow, StateA<'workflow>> {
    fn next(mut self: Box<Self>) -> Transition<'workflow, WorkflowResult> {
        self.message.push_str(self.state.0);
        self.message.push_str(self.my_str);

        self.transition(StateB)
    }
}

struct StateB;

impl State for StateB {
    fn name(&self) -> String {
        type_name::<StateB>().to_string()
    }
}

impl<'workflow> MyWorkflowState<'workflow> for MyWorkflow<'workflow, StateB> {
    fn next(mut self: Box<Self>) -> Transition<'workflow, WorkflowResult> {
        self.message.push_str(self.my_str_container.my_str);

        self.finish_with(|workflow| WorkflowResult {
            message: workflow.message,
        })
    }
}

struct WorkflowResult {
    message: String,
}

#[test]
fn test_basic_workflow_manual_impls() {
    let workflow = MyWorkflow::new(
        StateA("Hello "),
        "world",
        StrContainer { my_str: "!" },
        String::new(),
    );
    let result = workflow.run();
    assert_eq!(result.message, "Hello world!");
}
