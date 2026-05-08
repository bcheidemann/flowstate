use std::any::type_name;

use flowstate::prelude::*;

#[derive(Workflow)]
#[flowstate(result = MyWorkflowResult)]
struct MyWorkflow<State> {
    #[state]
    state: State,
    ctx: String,
}

#[derive(State)]
struct StateA;

impl MyWorkflowState for MyWorkflow<StateA> {
    fn next(self: Box<Self>) -> Transition<MyWorkflowResult> {
        self.transition(StateB)
    }
}

#[derive(State)]
#[flowstate(name = "This is state B!")]
struct StateB;

impl MyWorkflowState for MyWorkflow<StateB> {
    fn next(self: Box<Self>) -> Transition<MyWorkflowResult> {
        self.transition(StateC { random_number: 32 })
    }
}

#[derive(State)]
#[flowstate(name = self.name())]
struct StateC {
    random_number: u16,
}

impl StateC {
    fn name(&self) -> String {
        format!("{}({})", type_name::<StateC>(), self.random_number)
    }
}

impl MyWorkflowState for MyWorkflow<StateC> {
    fn next(self: Box<Self>) -> Transition<MyWorkflowResult> {
        self.transition(StateD)
    }
}

#[derive(State)]
struct StateD;

impl MyWorkflowState for MyWorkflow<StateD> {
    fn state_name(&self) -> String {
        format!("{} (ctx = {})", self.state.name(), self.ctx)
    }

    fn next(self: Box<Self>) -> Transition<MyWorkflowResult> {
        self.transition(StateB)
    }
}

type MyWorkflowResult = ();

#[test]
fn test_state_names_1() {
    let state_a_name = MyWorkflow::new(StateA, "ctx".into()).state_name();

    assert_eq!(state_a_name, "state_names::StateA");
}

#[test]
fn test_state_names_2() {
    let state_a_name = MyWorkflow::new(StateB, "ctx".into()).state_name();

    assert_eq!(state_a_name, "This is state B!");
}

#[test]
fn test_state_names_3() {
    let state_a_name = MyWorkflow::new(StateC { random_number: 42 }, "ctx".into()).state_name();

    assert_eq!(state_a_name, "state_names::StateC(42)");
}

#[test]
fn test_state_names_4() {
    let state_a_name = MyWorkflow::new(StateD, "some context".into()).state_name();

    assert_eq!(state_a_name, "state_names::StateD (ctx = some context)");
}
