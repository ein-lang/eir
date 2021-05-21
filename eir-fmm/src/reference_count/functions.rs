use super::{super::error::CompileError, pointers};
use crate::closures;

pub fn clone_function(
    builder: &fmm::build::InstructionBuilder,
    closure_pointer: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    pointers::clone_pointer(builder, closure_pointer)
}

pub fn drop_function(
    builder: &fmm::build::InstructionBuilder,
    closure_pointer: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    pointers::if_heap_pointer(builder, closure_pointer, |builder| {
        builder.if_(
            fmm::build::comparison_operation(
                fmm::ir::ComparisonOperator::Equal,
                builder.atomic_operation(
                    fmm::ir::AtomicOperator::Subtract,
                    pointers::get_counter_pointer(&builder, closure_pointer)?,
                    fmm::ir::Primitive::PointerInteger(1),
                )?,
                fmm::ir::Primitive::PointerInteger(0),
            )?,
            |builder| -> Result<_, CompileError> {
                builder.call(
                    closures::compile_load_drop_function(&builder, closure_pointer.clone())?,
                    vec![fmm::build::bit_cast(
                        fmm::types::Primitive::PointerInteger,
                        closure_pointer.clone(),
                    )
                    .into()],
                )?;

                builder.free_heap(fmm::build::bit_cast(
                    fmm::types::Pointer::new(fmm::types::Primitive::Integer8),
                    closure_pointer.clone(),
                ))?;

                Ok(builder.branch(fmm::build::VOID_VALUE.clone()))
            },
            |builder| Ok(builder.branch(fmm::build::VOID_VALUE.clone())),
        )?;

        Ok(())
    })?;

    Ok(())
}
