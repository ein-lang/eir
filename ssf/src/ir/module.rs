use super::declaration::Declaration;
use super::definition::Definition;
use super::foreign_declaration::ForeignDeclaration;
use super::foreign_definition::ForeignDefinition;
use super::variant_definition::VariantDefinition;
use crate::types::canonicalize;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    variant_definitions: Vec<VariantDefinition>,
    foreign_declarations: Vec<ForeignDeclaration>,
    foreign_definitions: Vec<ForeignDefinition>,
    declarations: Vec<Declaration>,
    definitions: Vec<Definition>,
}

impl Module {
    pub fn new(
        variant_definitions: Vec<VariantDefinition>,
        foreign_declarations: Vec<ForeignDeclaration>,
        foreign_definitions: Vec<ForeignDefinition>,
        declarations: Vec<Declaration>,
        definitions: Vec<Definition>,
    ) -> Self {
        Self {
            variant_definitions: variant_definitions
                .iter()
                .map(|definition| definition.convert_types(&canonicalize))
                .collect(),
            foreign_declarations: foreign_declarations
                .iter()
                .map(|declaration| declaration.convert_types(&canonicalize))
                .collect(),
            foreign_definitions,
            declarations: declarations
                .iter()
                .map(|declaration| declaration.convert_types(&canonicalize))
                .collect(),
            definitions: definitions
                .iter()
                .map(|definition| definition.convert_types(&canonicalize))
                .map(|definition| definition.infer_environment(&Default::default()))
                .collect(),
        }
    }

    pub fn variant_definitions(&self) -> &[VariantDefinition] {
        &self.variant_definitions
    }

    pub fn foreign_declarations(&self) -> &[ForeignDeclaration] {
        &self.foreign_declarations
    }

    pub fn foreign_definitions(&self) -> &[ForeignDefinition] {
        &self.foreign_definitions
    }

    pub fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }
}
