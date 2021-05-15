use super::super::error::CompileError;
use super::expressions;
use super::pointers;
use crate::types;
use std::collections::HashMap;

pub fn compile_variant_clone_function(
    module_builder: &fmm::build::ModuleBuilder,
    type_: &eir::types::Type,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(module_builder.define_function(
        format!("variant_clone_{}", types::compile_type_id(type_)),
        vec![fmm::ir::Argument::new(
            "_payload",
            types::compile_variant_payload(),
        )],
        |builder| -> Result<_, CompileError> {
            let payload = fmm::build::variable("_payload", types::compile_variant_payload());

            if crate::variants::is_payload_boxed(type_)? {
                pointers::clone_pointer(&builder, &payload)?;
            } else {
                expressions::clone_expression(
                    &builder,
                    &crate::variants::compile_unboxed_payload(&builder, &payload, type_, types)?,
                    type_,
                    types,
                )?;
            }

            Ok(builder.return_(fmm::build::VOID_VALUE.clone()))
        },
        fmm::build::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
    )?)
}

pub fn compile_variant_drop_function(
    module_builder: &fmm::build::ModuleBuilder,
    type_: &eir::types::Type,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(module_builder.define_function(
        format!("variant_drop_{}", types::compile_type_id(type_)),
        vec![fmm::ir::Argument::new(
            "_payload",
            types::compile_variant_payload(),
        )],
        |builder| -> Result<_, CompileError> {
            let payload = fmm::build::variable("_payload", types::compile_variant_payload());

            if crate::variants::is_payload_boxed(type_)? {
                pointers::drop_pointer(&builder, &payload)?;
            } else {
                expressions::drop_expression(
                    &builder,
                    &crate::variants::compile_unboxed_payload(&builder, &payload, type_, types)?,
                    type_,
                    types,
                )?;
            }

            Ok(builder.return_(fmm::build::VOID_VALUE.clone()))
        },
        fmm::build::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
    )?)
}
