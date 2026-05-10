use std::any::type_name;

use flowstate::{WorkflowState, prelude::*};

struct StrContainer<'a> {
    #[allow(unused)]
    my_str: &'a str,
}

#[derive(Workflow)]
#[flowstate(
    result = WorkflowResult,
    state_trait = MyWorkflowState,
)]
struct MyWorkflow<'workflow, State> {
    #[state]
    state: State,
    my_str: &'workflow str,
    my_str_container: StrContainer<'workflow>,
    message: String,
}

struct StateA<'a>(&'a str);

// TODO: Update derive macro for state to handle generic lifetimes
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

#[derive(State)]
struct StateB;

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
