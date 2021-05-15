use crate::types;

pub const TYPE_INFORMATION_CLONE_FUNCTION_ELEMENT_INDEX: usize = 0;
pub const TYPE_INFORMATION_DROP_FUNCTION_ELEMENT_INDEX: usize = 1;

pub fn compile_type_information_global_variable(
    module_builder: &fmm::build::ModuleBuilder,
    type_: &eir::types::Type,
) -> Result<(), fmm::build::BuildError> {
    // TODO Define GC functions.
    module_builder.define_variable(
        types::compile_type_id(type_),
        fmm::build::record(vec![
            fmm::ir::Undefined::new(fmm::types::Function::new(
                vec![fmm::types::Primitive::PointerInteger.into()],
                fmm::build::VOID_TYPE.clone(),
                fmm::types::CallingConvention::Target,
            ))
            .into(),
            fmm::ir::Undefined::new(fmm::types::Function::new(
                vec![fmm::types::Primitive::PointerInteger.into()],
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
