use super::super::error::CompileError;

pub(super) const COUNTER_TYPE: fmm::types::Primitive = fmm::types::Primitive::PointerInteger;

pub fn allocate_heap(
    builder: &fmm::build::InstructionBuilder,
    type_: impl Into<fmm::types::Type>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.record_address(
        builder.allocate_heap(fmm::types::Record::new(vec![
            COUNTER_TYPE.into(),
            type_.into(),
        ])),
        1,
    )?)
}
