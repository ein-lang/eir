use super::expression::Expression;
use super::variable::Variable;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct DropVariable {
    variable: Variable,
    expression: Arc<Expression>,
}

impl DropVariable {
    pub fn new(variable: Variable, expression: impl Into<Expression>) -> Self {
        Self {
            variable,
            expression: expression.into().into(),
        }
    }

    pub fn variable(&self) -> &Variable {
        &self.variable
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
