use crate::{error::CompileError, types};
use std::collections::HashMap;

pub fn get_record_element(
    builder: &fmm::build::InstructionBuilder,
    record: &fmm::build::TypedExpression,
    type_: &eir::types::Record,
    element_index: usize,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.deconstruct_record(
        if types::is_record_boxed(type_, types) {
            builder.load(fmm::build::bit_cast(
                fmm::types::Pointer::new(types::compile_unboxed_record(type_, types)),
                record.clone(),
            ))?
        } else {
            record.clone()
        },
        element_index,
    )?)
}
