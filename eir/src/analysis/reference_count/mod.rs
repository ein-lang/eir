use crate::ir::*;

pub fn count_references(module: &Module) -> Module {
    Module::new(
        module.type_definitions().to_vec(),
        module.foreign_declarations().to_vec(),
        module.foreign_definitions().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(convert_definition)
            .collect(),
    )
}

fn convert_definition(definition: &Definition) -> Definition {
    todo!()
}
