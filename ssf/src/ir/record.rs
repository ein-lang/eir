use super::expression::Expression;
use crate::types::{self, Type};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    type_: types::Record,
    elements: Vec<Expression>,
}

impl Record {
    pub fn new(record_type: types::Record, elements: Vec<Expression>) -> Self {
        Self {
            type_: record_type,
            elements,
        }
    }

    pub fn type_(&self) -> &types::Record {
        &self.type_
    }

    pub fn elements(&self) -> &[Expression] {
        &self.elements
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        self.elements
            .iter()
            .flat_map(|element| element.find_variables())
            .collect()
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        Self::new(
            self.type_.clone(),
            self.elements
                .iter()
                .map(|element| element.infer_environment(variables))
                .collect(),
        )
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        Self::new(
            convert(&self.type_.clone().into()).into_record().unwrap(),
            self.elements
                .iter()
                .map(|element| element.convert_types(convert))
                .collect(),
        )
    }
}
