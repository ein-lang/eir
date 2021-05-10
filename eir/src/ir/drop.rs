use super::expression::Expression;
use crate::types::Type;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Clone {
    variable: Variable,
}

impl Clone {
    pub fn new(variable: Variable) -> Self {
        Self { variable }
    }

    pub fn variable(&self) -> &Variable {
        &self.variable
    }
}
