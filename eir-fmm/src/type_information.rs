use crate::{error::CompileError, reference_count, types};
use std::collections::HashMap;

pub const TYPE_INFORMATION_CLONE_FUNCTION_ELEMENT_INDEX: usize = 0;
pub const TYPE_INFORMATION_DROP_FUNCTION_ELEMENT_INDEX: usize = 1;

pub fn compile(
    module_builder: &fmm::build::ModuleBuilder,
    type_: &eir::types::Type,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<(), CompileError> {
    module_builder.define_variable(
        types::compile_type_id(type_),
        fmm::build::record(vec![
            reference_count::compile_variant_clone_function(module_builder, type_, types)?,
            reference_count::compile_variant_drop_function(module_builder, type_, types)?,
        ]),
        fmm::ir::VariableDefinitionOptions::new().set_linkage(fmm::ir::Linkage::Weak),
    );

    Ok(())
}
