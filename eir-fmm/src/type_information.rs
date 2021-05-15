use crate::error::CompileError;
use crate::reference_count;
use crate::types;
use std::collections::HashMap;

pub const TYPE_INFORMATION_CLONE_FUNCTION_ELEMENT_INDEX: usize = 0;
pub const TYPE_INFORMATION_DROP_FUNCTION_ELEMENT_INDEX: usize = 1;

pub fn compile_type_information_global_variable(
    module_builder: &fmm::build::ModuleBuilder,
    type_: &eir::types::Type,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<(), CompileError> {
    // TODO Define GC functions.
    module_builder.define_variable(
        types::compile_type_id(type_),
        fmm::build::record(vec![
            reference_count::compile_variant_clone_function(module_builder, type_, types)?.into(),
            fmm::ir::Undefined::new(fmm::types::Function::new(
                vec![types::compile_variant_payload().into()],
                fmm::build::VOID_TYPE.clone(),
                fmm::types::CallingConvention::Target,
            ))
            .into(),
        ]),
        false,
        fmm::ir::Linkage::Weak,
    );

    Ok(())
}
