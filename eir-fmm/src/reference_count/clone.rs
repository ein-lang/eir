use super::{super::error::CompileError, utilities};
use std::collections::HashMap;

const TYPE_INFORMATION_CLONE_FUNCTION_ELEMENT_INDEX: usize = 0;

pub fn clone_expression(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    type_: &eir::types::Type,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<(), CompileError> {
    match type_ {
        eir::types::Type::ByteString => {
            clone_pointer(builder, &builder.deconstruct_record(expression.clone(), 0)?)?
        }
        eir::types::Type::Function(_) => todo!(),
        eir::types::Type::Record(record) => {
            builder.call(
                fmm::build::variable(
                    utilities::get_record_clone_function_name(record),
                    utilities::create_record_rc_function_type(record, types),
                ),
                vec![expression.clone()],
            )?;
        }
        eir::types::Type::Variant => {
            builder.call(
                builder.deconstruct_record(
                    builder.load(builder.deconstruct_record(expression.clone(), 0)?)?,
                    TYPE_INFORMATION_CLONE_FUNCTION_ELEMENT_INDEX,
                )?,
                vec![builder.deconstruct_record(expression.clone(), 1)?],
            )?;
        }
        eir::types::Type::Boolean | eir::types::Type::Number => {}
    }

    Ok(())
}

fn clone_pointer(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    utilities::if_heap_pointer(builder, expression, |builder| {
        builder.atomic_operation(
            fmm::ir::AtomicOperator::Add,
            utilities::get_counter_pointer(&builder, expression)?,
            fmm::ir::Primitive::PointerInteger(1),
        )?;

        Ok(())
    })?;

    Ok(())
}
