use super::variable::Variable;

#[derive(Clone, Debug, PartialEq)]
pub struct CloneVariable {
    variable: Variable,
}

impl CloneVariable {
    pub fn new(variable: Variable) -> Self {
        Self { variable }
    }

    pub fn variable(&self) -> &Variable {
        &self.variable
    }
}
