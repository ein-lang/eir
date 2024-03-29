use super::{super::error::CompileError, expression};
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
        fmm::types::void_type(),
        |builder| -> Result<_, CompileError> {
            let payload = fmm::build::variable("_payload", types::compile_variant_payload());

            expression::clone_expression(
                &builder,
                &crate::variant::compile_unboxed_payload(&builder, &payload, type_, types)?,
                type_,
                types,
            )?;

            Ok(builder.return_(fmm::ir::void_value()))
        },
        function_definition_options(),
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
        fmm::types::void_type(),
        |builder| -> Result<_, CompileError> {
            let payload = fmm::build::variable("_payload", types::compile_variant_payload());

            expression::drop_expression(
                &builder,
                &crate::variant::compile_unboxed_payload(&builder, &payload, type_, types)?,
                type_,
                types,
            )?;

            Ok(builder.return_(fmm::ir::void_value()))
        },
        function_definition_options(),
    )
}

fn function_definition_options() -> fmm::ir::FunctionDefinitionOptions {
    fmm::ir::FunctionDefinitionOptions::new()
        .set_calling_convention(fmm::types::CallingConvention::Target)
        .set_linkage(fmm::ir::Linkage::Weak)
}
