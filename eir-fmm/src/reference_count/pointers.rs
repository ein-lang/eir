use super::super::error::CompileError;

pub fn clone_pointer(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    if_heap_pointer(builder, expression, |builder| {
        builder.atomic_operation(
            fmm::ir::AtomicOperator::Add,
            get_counter_pointer(&builder, expression)?,
            fmm::ir::Primitive::PointerInteger(1),
        )?;

        Ok(())
    })?;

    Ok(())
}

pub fn drop_pointer(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    if_heap_pointer(builder, expression, |builder| {
        builder.if_(
            fmm::build::comparison_operation(
                fmm::ir::ComparisonOperator::Equal,
                builder.atomic_operation(
                    fmm::ir::AtomicOperator::Subtract,
                    get_counter_pointer(&builder, expression)?,
                    fmm::ir::Primitive::PointerInteger(1),
                )?,
                fmm::ir::Primitive::PointerInteger(0),
            )?,
            |builder| -> Result<_, CompileError> {
                builder.free_heap(expression.clone())?;

                Ok(builder.branch(fmm::build::VOID_VALUE.clone()))
            },
            |builder| Ok(builder.branch(fmm::build::VOID_VALUE.clone())),
        )?;

        Ok(())
    })?;

    Ok(())
}

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

fn if_heap_pointer(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
    then: impl Fn(&fmm::build::InstructionBuilder) -> Result<(), CompileError>,
) -> Result<(), CompileError> {
    // TODO Remove a null pointer check?
    builder.if_(
        fmm::build::comparison_operation(
            fmm::ir::ComparisonOperator::NotEqual,
            fmm::build::bit_cast(fmm::types::Primitive::PointerInteger, pointer.clone()),
            fmm::ir::Undefined::new(fmm::types::Primitive::PointerInteger),
        )?,
        |builder| -> Result<_, CompileError> {
            builder.if_(
                is_heap_pointer(pointer)?,
                |builder| -> Result<_, CompileError> {
                    then(&builder)?;
                    Ok(builder.branch(fmm::build::VOID_VALUE.clone()))
                },
                |builder| Ok(builder.branch(fmm::build::VOID_VALUE.clone())),
            )?;
            Ok(builder.branch(fmm::build::VOID_VALUE.clone()))
        },
        |builder| Ok(builder.branch(fmm::build::VOID_VALUE.clone())),
    )?;

    Ok(())
}

fn is_heap_pointer(
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

fn get_counter_pointer(
    builder: &fmm::build::InstructionBuilder,
    heap_pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    builder.pointer_address(
        fmm::build::bit_cast(
            fmm::types::Pointer::new(fmm::types::Primitive::PointerInteger),
            heap_pointer.clone(),
        ),
        fmm::ir::Primitive::PointerInteger(-1),
    )
}
