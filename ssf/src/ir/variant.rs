use super::expression::Expression;
use crate::types::Type;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Variant {
    name: String,
    payload: Arc<Expression>,
}

impl Variant {
    pub fn new(name: impl Into<String>, payload: impl Into<Expression>) -> Self {
        Self {
            name: name.into(),
            payload: payload.into().into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn payload(&self) -> &Expression {
        &self.payload
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        self.payload.find_variables()
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self::new(self.name.clone(), self.payload.infer_environment(variables))
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self::new(self.name.clone(), self.payload.convert_types(convert))
    }
}
