use super::expression::Expression;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct DropVariable {
    variables: HashSet<String>,
    expression: Arc<Expression>,
}

impl DropVariable {
    pub fn new(variables: HashSet<String>, expression: impl Into<Expression>) -> Self {
        Self {
            variables,
            expression: expression.into().into(),
        }
    }

    pub fn variables(&self) -> &HashSet<String> {
        &self.variables
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
