use super::super::error::CompileError;

pub fn get_raw_pointer(
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::bitwise_operation(
        fmm::ir::BitwiseOperator::Xor,
        fmm::build::bit_cast(fmm::types::Primitive::PointerInteger, pointer.clone()),
        fmm::ir::Primitive::PointerInteger(1),
    )?
    .into())
}

pub fn is_heap_pointer(
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::comparison_operation(
        fmm::ir::ComparisonOperator::NotEqual,
        fmm::build::bitwise_operation(
            fmm::ir::BitwiseOperator::And,
            fmm::build::bit_cast(fmm::types::Primitive::PointerInteger, pointer.clone()),
            fmm::ir::Primitive::PointerInteger(1),
        )?,
        fmm::ir::Primitive::PointerInteger(1),
    )?
    .into())
}
