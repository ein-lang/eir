use super::{super::error::CompileError, expressions};
use crate::types;
use std::collections::HashMap;

pub fn compile_variant_clone_function(
    module_builder: &fmm::build::ModuleBuilder,
    type_: &eir::types::Type,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    module_builder.define_function(
        format!("variant_clone_{}", types::compile_type_id(type_)),
        vec![fmm::ir::Argument::new(
            "_payload",
            types::compile_variant_payload(),
        )],
        |builder| -> Result<_, CompileError> {
            let payload = fmm::build::variable("_payload", types::compile_variant_payload());

            expressions::clone_expression(
                &builder,
                &compile_payload(&builder, &payload, type_, types)?,
                type_,
                types,
            )?;

            Ok(builder.return_(fmm::ir::VOID_VALUE.clone()))
        },
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
    )
}

pub fn compile_variant_drop_function(
    module_builder: &fmm::build::ModuleBuilder,
    type_: &eir::types::Type,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    module_builder.define_function(
        format!("variant_drop_{}", types::compile_type_id(type_)),
        vec![fmm::ir::Argument::new(
            "_payload",
            types::compile_variant_payload(),
        )],
        |builder| -> Result<_, CompileError> {
            let payload = fmm::build::variable("_payload", types::compile_variant_payload());

            expressions::drop_expression(
                &builder,
                &compile_payload(&builder, &payload, type_, types)?,
                type_,
                types,
            )?;

            Ok(builder.return_(fmm::ir::VOID_VALUE.clone()))
        },
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
    )
}

fn compile_payload(
    builder: &fmm::build::InstructionBuilder,
    payload: &fmm::build::TypedExpression,
    type_: &eir::types::Type,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(crate::variants::compile_union_bit_cast(
        builder,
        types::compile(type_, types),
        payload.clone(),
    )?)
}
