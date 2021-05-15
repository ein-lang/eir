use super::{super::error::CompileError, pointers, records};
use crate::{
    type_information::TYPE_INFORMATION_DROP_FUNCTION_ELEMENT_INDEX,
    types,
    variants::{VARIANT_PAYLOAD_ELEMENT_INDEX, VARIANT_TAG_ELEMENT_INDEX},
};
use std::collections::HashMap;

pub fn drop_expression(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    type_: &eir::types::Type,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<(), CompileError> {
    match type_ {
        eir::types::Type::ByteString => {
            drop_pointer(builder, &builder.deconstruct_record(expression.clone(), 0)?)?
        }
        eir::types::Type::Function(_) => todo!(),
        eir::types::Type::Record(record) => {
            builder.call(
                fmm::build::variable(
                    records::get_record_drop_function_name(record.name()),
                    records::compile_record_rc_function_type(record, types),
                ),
                vec![expression.clone()],
            )?;
        }
        eir::types::Type::Variant => {
            builder.call(
                builder.deconstruct_record(
                    builder.load(
                        builder
                            .deconstruct_record(expression.clone(), VARIANT_TAG_ELEMENT_INDEX)?,
                    )?,
                    TYPE_INFORMATION_DROP_FUNCTION_ELEMENT_INDEX,
                )?,
                vec![builder
                    .deconstruct_record(expression.clone(), VARIANT_PAYLOAD_ELEMENT_INDEX)?],
            )?;
        }
        eir::types::Type::Boolean | eir::types::Type::Number => {}
    }

    Ok(())
}

pub fn compile_record_drop_function(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &eir::ir::TypeDefinition,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<(), CompileError> {
    let record_type = eir::types::Record::new(definition.name());
    let fmm_record_type = types::compile_record(&eir::types::Record::new(definition.name()), types);

    module_builder.define_function(
        records::get_record_drop_function_name(definition.name()),
        vec![fmm::ir::Argument::new("record", fmm_record_type.clone())],
        |builder| -> Result<_, CompileError> {
            for (index, type_) in definition.type_().elements().iter().enumerate() {
                drop_expression(
                    &builder,
                    &crate::records::get_record_element(
                        &builder,
                        &fmm::build::variable("record", fmm_record_type.clone()),
                        &record_type,
                        index,
                        types,
                    )?,
                    type_,
                    types,
                )?;
            }

            Ok(builder.return_(fmm::build::VOID_VALUE.clone()))
        },
        fmm::build::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
    )?;

    Ok(())
}

fn drop_pointer(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    pointers::if_heap_pointer(builder, expression, |builder| {
        builder.if_(
            fmm::build::comparison_operation(
                fmm::ir::ComparisonOperator::Equal,
                builder.atomic_operation(
                    fmm::ir::AtomicOperator::Subtract,
                    pointers::get_counter_pointer(&builder, expression)?,
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
