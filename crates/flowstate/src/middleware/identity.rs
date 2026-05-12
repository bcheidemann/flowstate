#[cfg(feature = "async")]
use crate::middleware::AsyncWorkflowMiddleware;
use crate::middleware::WorkflowMiddleware;

#[derive(Default)]
pub struct IdentityMiddleware;

impl WorkflowMiddleware for IdentityMiddleware {}

#[cfg(feature = "async")]
impl AsyncWorkflowMiddleware for IdentityMiddleware {}
