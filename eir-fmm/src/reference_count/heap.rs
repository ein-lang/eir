use super::super::error::CompileError;

pub(super) const COUNTER_TYPE: fmm::types::Primitive = fmm::types::Primitive::PointerInteger;
pub(super) const INITIAL_COUNTER: usize = 0;

pub fn allocate_heap(
    builder: &fmm::build::InstructionBuilder,
    type_: impl Into<fmm::types::Type>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let pointer = builder.allocate_heap(fmm::types::Record::new(vec![
        COUNTER_TYPE.into(),
        type_.into(),
    ]));

    builder.store(
        fmm::ir::Primitive::PointerInteger(INITIAL_COUNTER as i64),
        builder.record_address(pointer.clone(), 0)?,
    );

    Ok(builder.record_address(pointer, 1)?)
}

pub fn free_heap(
    builder: &fmm::build::InstructionBuilder,
    pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<(), CompileError> {
    builder.free_heap(builder.pointer_address(
        fmm::build::bit_cast(
            fmm::types::Pointer::new(fmm::types::Primitive::PointerInteger),
            pointer,
        ),
        fmm::ir::Primitive::PointerInteger(-1),
    )?)?;

    Ok(())
}