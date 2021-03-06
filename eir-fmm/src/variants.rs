use crate::{types, CompileError};
use std::collections::HashMap;

pub const VARIANT_TAG_ELEMENT_INDEX: usize = 0;
pub const VARIANT_PAYLOAD_ELEMENT_INDEX: usize = 1;

pub fn compile_tag(type_: &eir::types::Type) -> fmm::build::TypedExpression {
    fmm::build::variable(types::compile_type_id(type_), types::compile_variant_tag())
}

pub fn compile_boxed_payload(
    builder: &fmm::build::InstructionBuilder,
    payload: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(compile_union_bit_cast(
        builder,
        types::compile_variant_payload(),
        payload.clone(),
    )?)
}

pub fn compile_unboxed_payload(
    builder: &fmm::build::InstructionBuilder,
    payload: &fmm::build::TypedExpression,
    type_: &eir::types::Type,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(compile_union_bit_cast(
        builder,
        types::compile(type_, types),
        payload.clone(),
    )?)
}

fn compile_union_bit_cast(
    builder: &fmm::build::InstructionBuilder,
    to_type: impl Into<fmm::types::Type>,
    argument: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let argument = argument.into();
    let to_type = to_type.into();

    Ok(if argument.type_() == &to_type {
        argument
    } else {
        builder.deconstruct_union(
            fmm::ir::Union::new(
                fmm::types::Union::new(vec![argument.type_().clone(), to_type]),
                0,
                argument.expression().clone(),
            ),
            1,
        )?
    })
}
