use super::super::types;
use super::{super::error::CompileError, pointers, record_utilities};
use crate::{
    type_information::TYPE_INFORMATION_CLONE_FUNCTION_ELEMENT_INDEX,
    variants::{VARIANT_PAYLOAD_ELEMENT_INDEX, VARIANT_TAG_ELEMENT_INDEX},
};
use std::collections::HashMap;

pub fn clone_expression(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    type_: &eir::types::Type,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<(), CompileError> {
    match type_ {
        eir::types::Type::ByteString => {
            pointers::clone_pointer(builder, &builder.deconstruct_record(expression.clone(), 0)?)?
        }
        eir::types::Type::Function(_) => todo!(),
        eir::types::Type::Record(record) => {
            builder.call(
                fmm::build::variable(
                    record_utilities::get_record_clone_function_name(record.name()),
                    record_utilities::compile_record_rc_function_type(record, types),
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
                    TYPE_INFORMATION_CLONE_FUNCTION_ELEMENT_INDEX,
                )?,
                vec![builder
                    .deconstruct_record(expression.clone(), VARIANT_PAYLOAD_ELEMENT_INDEX)?],
            )?;
        }
        eir::types::Type::Boolean | eir::types::Type::Number => {}
    }

    Ok(())
}

pub fn compile_record_clone_function(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &eir::ir::TypeDefinition,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<(), CompileError> {
    let record_type = eir::types::Record::new(definition.name());
    let fmm_record_type = types::compile_record(&record_type, types);

    module_builder.define_function(
        record_utilities::get_record_clone_function_name(definition.name()),
        vec![fmm::ir::Argument::new("record", fmm_record_type.clone())],
        |builder| -> Result<_, CompileError> {
            for (index, type_) in definition.type_().elements().iter().enumerate() {
                clone_expression(
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
