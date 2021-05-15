use super::super::error::CompileError;
use super::super::types;
use super::pointers;
use std::collections::hash_map::HashMap;

pub fn if_heap_pointer(
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
                pointers::is_heap_pointer(pointer)?,
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

pub fn get_counter_pointer(
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

pub fn get_record_clone_function_name(record: &eir::types::Record) -> String {
    format!("eir_clone_{}", record.name())
}

pub fn get_record_drop_function_name(record: &eir::types::Record) -> String {
    format!("eir_drop_{}", record.name())
}

pub fn create_record_rc_function_type(
    record: &eir::types::Record,
    types: &HashMap<String, eir::types::RecordBody>,
) -> fmm::types::Function {
    fmm::types::Function::new(
        vec![types::compile_record(record, types)],
        fmm::build::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
    )
}
