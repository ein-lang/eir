use super::expression::Expression;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct VariantAlternative {
    tag: String,
    type_: Type,
    name: String,
    expression: Expression,
}

impl VariantAlternative {
    pub fn new(
        tag: impl Into<String>,
        type_: impl Into<Type>,
        name: impl Into<String>,
        expression: impl Into<Expression>,
    ) -> Self {
        Self {
            tag: tag.into(),
            type_: type_.into(),
            name: name.into(),
            expression: expression.into(),
        }
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        let mut variables = self.expression.find_variables();

        variables.remove(&self.name);

        variables
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        let mut variables = variables.clone();

        variables.insert(self.name.clone(), self.type_.clone());

        Self {
            tag: self.tag.clone(),
            type_: self.type_.clone(),
            name: self.name.clone(),
            expression: self.expression.infer_environment(&variables),
        }
    }
}
