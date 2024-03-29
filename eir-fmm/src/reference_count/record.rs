use super::{
    super::{error::CompileError, types},
    expression, pointer, record_utilities, reference_count_function_definition_options,
};
use std::collections::HashMap;

const ARGUMENT_NAME: &str = "_record";

pub fn compile_record_clone_function(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &eir::ir::TypeDefinition,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<(), CompileError> {
    let record_type = eir::types::Record::new(definition.name());
    let fmm_record_type = types::compile_record(&record_type, types);

    module_builder.define_function(
        record_utilities::get_record_clone_function_name(definition.name()),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            fmm_record_type.clone(),
        )],
        fmm::types::void_type(),
        |builder| -> Result<_, CompileError> {
            let record = fmm::build::variable(ARGUMENT_NAME, fmm_record_type.clone());

            if types::is_record_boxed(&record_type, types) {
                pointer::clone_pointer(&builder, &record)?;
            } else {
                for (index, type_) in definition.type_().elements().iter().enumerate() {
                    expression::clone_expression(
                        &builder,
                        &crate::records::get_record_element(
                            &builder,
                            &record,
                            &record_type,
                            index,
                            types,
                        )?,
                        type_,
                        types,
                    )?;
                }
            }

            Ok(builder.return_(fmm::ir::void_value()))
        },
        reference_count_function_definition_options(),
    )?;

    Ok(())
}

pub fn compile_record_drop_function(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &eir::ir::TypeDefinition,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<(), CompileError> {
    let record_type = eir::types::Record::new(definition.name());
    let fmm_record_type = types::compile_record(&record_type, types);

    module_builder.define_function(
        record_utilities::get_record_drop_function_name(definition.name()),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            fmm_record_type.clone(),
        )],
        fmm::types::void_type(),
        |builder| -> Result<_, CompileError> {
            let record = fmm::build::variable(ARGUMENT_NAME, fmm_record_type.clone());

            if types::is_record_boxed(&record_type, types) {
                pointer::drop_pointer(&builder, &record, |builder| {
                    drop_record_elements(
                        builder,
                        &record,
                        &record_type,
                        definition.type_(),
                        types,
                    )?;

                    Ok(())
                })?;
            } else {
                drop_record_elements(&builder, &record, &record_type, definition.type_(), types)?;
            }

            Ok(builder.return_(fmm::ir::void_value()))
        },
        reference_count_function_definition_options(),
    )?;

    Ok(())
}

fn drop_record_elements(
    builder: &fmm::build::InstructionBuilder,
    record: &fmm::build::TypedExpression,
    record_type: &eir::types::Record,
    record_body_type: &eir::types::RecordBody,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<(), CompileError> {
    for (index, type_) in record_body_type.elements().iter().enumerate() {
        expression::drop_expression(
            builder,
            &crate::records::get_record_element(builder, record, record_type, index, types)?,
            type_,
            types,
        )?;
    }

    Ok(())
}
