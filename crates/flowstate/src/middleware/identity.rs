use crate::middleware::AsyncWorkflowMiddleware;

#[derive(Default)]
pub struct IdentityMiddleware;

impl AsyncWorkflowMiddleware for IdentityMiddleware {}
