use super::super::types;
use super::{super::error::CompileError, record_utilities};
use super::{expressions, pointers};
use std::collections::HashMap;

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
            let argument = fmm::build::variable("record", fmm_record_type.clone());

            if types::is_record_boxed(&record_type, types) {
                pointers::clone_pointer(&builder, &argument)?;
            } else {
                for (index, type_) in definition.type_().elements().iter().enumerate() {
                    expressions::clone_expression(
                        &builder,
                        &crate::records::get_record_element(
                            &builder,
                            &argument,
                            &record_type,
                            index,
                            types,
                        )?,
                        type_,
                        types,
                    )?;
                }
            }

            Ok(builder.return_(fmm::build::VOID_VALUE.clone()))
        },
        fmm::build::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
    )?;

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
        record_utilities::get_record_drop_function_name(definition.name()),
        vec![fmm::ir::Argument::new("record", fmm_record_type.clone())],
        |builder| -> Result<_, CompileError> {
            for (index, type_) in definition.type_().elements().iter().enumerate() {
                expressions::drop_expression(
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
