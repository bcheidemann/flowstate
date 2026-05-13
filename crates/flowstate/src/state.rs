#[cfg(feature = "async")]
use crate::AsyncContext;
use crate::Context;

pub trait State {
    fn name(&self) -> String;
    fn record_context(&self, _ctx: &mut Context) {}
}

#[cfg(feature = "async")]
pub trait AsyncState {
    fn name(&self) -> String;
    fn record_context(&self, _ctx: &mut AsyncContext) {}
}
