use super::expression::Expression;
use crate::types::Type;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Clone {
    expression: Arc<Expression>,
}

impl Clone {
    pub fn new(expression: impl Into<Expression>) -> Self {
        Self {
            expression: expression.into().into(),
        }
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
