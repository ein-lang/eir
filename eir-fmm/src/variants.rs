use crate::{reference_count, types, CompileError};
use std::collections::HashMap;

pub const VARIANT_TAG_ELEMENT_INDEX: usize = 0;
pub const VARIANT_PAYLOAD_ELEMENT_INDEX: usize = 1;

pub fn compile_tag(type_: &eir::types::Type) -> fmm::build::TypedExpression {
    fmm::build::variable(types::compile_type_id(type_), types::compile_variant_tag())
}

pub fn compile_boxed_payload(
    builder: &fmm::build::InstructionBuilder,
    payload: &fmm::build::TypedExpression,
    type_: &eir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(compile_union_bit_cast(
        builder,
        types::compile_variant_payload(),
        // Strings have two words.
        if is_payload_boxed(type_)? {
            let pointer = reference_count::allocate_heap(builder, payload.type_().clone())?;

            builder.store(payload.clone(), pointer.clone());

            pointer
        } else {
            payload.clone()
        },
    )?)
}

pub fn compile_unboxed_payload(
    builder: &fmm::build::InstructionBuilder,
    payload: &fmm::build::TypedExpression,
    type_: &eir::types::Type,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(if is_payload_boxed(type_)? {
        // Do small optimization of moving payload directly instead of cloning a payload and dropping a variant.
        let pointer = fmm::build::bit_cast(
            fmm::types::Pointer::new(types::compile(type_, types)),
            payload.clone(),
        );
        let value = builder.load(pointer.clone())?;

        reference_count::drop_pointer(builder, &pointer.into(), |_| Ok(()))?;

        value
    } else {
        compile_union_bit_cast(builder, types::compile(type_, types), payload.clone())?
    })
}

pub fn is_payload_boxed(type_: &eir::types::Type) -> Result<bool, CompileError> {
    match type_ {
        eir::types::Type::ByteString => Ok(true),
        eir::types::Type::Variant => Err(CompileError::NestedVariant),
        eir::types::Type::Boolean
        | eir::types::Type::Function(_)
        | eir::types::Type::Number
        | eir::types::Type::Record(_) => Ok(false),
    }
}

pub fn compile_union_bit_cast(
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
