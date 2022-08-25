use super::error::CompileError;
use crate::{closure, entry_function, expression, types};
use std::collections::HashMap;

pub fn compile(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &eir::ir::Definition,
    global_variables: &HashMap<String, fmm::build::TypedExpression>,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<(), CompileError> {
    module_builder.define_variable(
        definition.name(),
        fmm::build::record(vec![
            entry_function::compile(module_builder, definition, global_variables, types)?,
            closure::compile_drop_function(module_builder, definition, types)?,
            expression::compile_arity(definition.arguments().iter().count()).into(),
            fmm::ir::Undefined::new(types::compile_closure_payload(definition, types)).into(),
        ]),
        fmm::ir::VariableDefinitionOptions::new()
            .set_linkage(fmm::ir::Linkage::External)
            .set_mutable(definition.is_thunk()),
    );

    Ok(())
}
