use super::type_::Type;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RecordContent {
    elements: Vec<Type>,
}

impl RecordContent {
    pub const fn new(elements: Vec<Type>) -> Self {
        RecordContent { elements }
    }

    pub fn elements(&self) -> &[Type] {
        &self.elements
    }
}
