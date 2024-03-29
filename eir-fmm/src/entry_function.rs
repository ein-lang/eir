use super::error::CompileError;
use crate::{closure, expression, reference_count, types};
use std::collections::HashMap;

const CLOSURE_NAME: &str = "_closure";

pub fn compile(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &eir::ir::Definition,
    variables: &HashMap<String, fmm::build::TypedExpression>,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(if definition.is_thunk() {
        compile_thunk(module_builder, definition, variables, types)?
    } else {
        compile_non_thunk(module_builder, definition, variables, types)?
    })
}

fn compile_non_thunk(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &eir::ir::Definition,
    variables: &HashMap<String, fmm::build::TypedExpression>,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    module_builder.define_anonymous_function(
        compile_arguments(definition, types),
        types::compile(definition.result_type(), types),
        |instruction_builder| {
            Ok(instruction_builder.return_(compile_body(
                module_builder,
                &instruction_builder,
                definition,
                variables,
                types,
            )?))
        },
        fmm::ir::FunctionDefinitionOptions::new()
            .set_calling_convention(fmm::types::CallingConvention::Source),
    )
}

fn compile_thunk(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &eir::ir::Definition,
    variables: &HashMap<String, fmm::build::TypedExpression>,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    compile_initial_thunk_entry(
        module_builder,
        definition,
        compile_normal_thunk_entry(module_builder, definition, types)?,
        compile_locked_thunk_entry(module_builder, definition, types)?,
        variables,
        types,
    )
}

fn compile_body(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    definition: &eir::ir::Definition,
    variables: &HashMap<String, fmm::build::TypedExpression>,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let payload_pointer = compile_payload_pointer(definition, types)?;
    let environment_pointer = if definition.is_thunk() {
        fmm::build::union_address(payload_pointer, 0)?.into()
    } else {
        payload_pointer
    };

    expression::compile(
        module_builder,
        instruction_builder,
        definition.body(),
        &variables
            .clone()
            .into_iter()
            .chain(
                definition
                    .environment()
                    .iter()
                    .enumerate()
                    .map(|(index, free_variable)| -> Result<_, CompileError> {
                        let value = instruction_builder.load(fmm::build::record_address(
                            environment_pointer.clone(),
                            index,
                        )?)?;

                        reference_count::clone_expression(
                            instruction_builder,
                            &value,
                            free_variable.type_(),
                            types,
                        )?;

                        Ok((free_variable.name().into(), value))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .chain(vec![(
                definition.name().into(),
                compile_closure_pointer(definition.type_(), types)?,
            )])
            .chain(definition.arguments().iter().map(|argument| {
                (
                    argument.name().into(),
                    fmm::build::variable(argument.name(), types::compile(argument.type_(), types)),
                )
            }))
            .collect(),
        types,
    )
}

fn compile_initial_thunk_entry(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &eir::ir::Definition,
    normal_entry_function: fmm::build::TypedExpression,
    lock_entry_function: fmm::build::TypedExpression,
    variables: &HashMap<String, fmm::build::TypedExpression>,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let entry_function_name = module_builder.generate_name();
    let entry_function_type = types::compile_entry_function(definition, types);
    let arguments = compile_arguments(definition, types);

    module_builder.define_function(
        &entry_function_name,
        arguments.clone(),
        types::compile(definition.result_type(), types),
        |instruction_builder| {
            let entry_function_pointer = compile_entry_function_pointer(definition, types)?;

            instruction_builder.if_(
                instruction_builder.compare_and_swap(
                    entry_function_pointer.clone(),
                    fmm::build::variable(&entry_function_name, entry_function_type.clone()),
                    lock_entry_function.clone(),
                    fmm::ir::AtomicOrdering::Acquire,
                    fmm::ir::AtomicOrdering::Relaxed,
                ),
                |instruction_builder| -> Result<_, CompileError> {
                    let value = compile_body(
                        module_builder,
                        &instruction_builder,
                        definition,
                        variables,
                        types,
                    )?;

                    reference_count::clone_expression(
                        &instruction_builder,
                        &value,
                        definition.result_type(),
                        types,
                    )?;

                    instruction_builder.store(
                        value.clone(),
                        compile_thunk_value_pointer(definition, types)?,
                    );

                    instruction_builder.store(
                        closure::compile_normal_thunk_drop_function(
                            module_builder,
                            definition,
                            types,
                        )?,
                        compile_drop_function_pointer(definition, types)?,
                    );

                    instruction_builder.atomic_store(
                        normal_entry_function.clone(),
                        entry_function_pointer.clone(),
                        fmm::ir::AtomicOrdering::Release,
                    );

                    Ok(instruction_builder.return_(value))
                },
                |instruction_builder| {
                    Ok(instruction_builder.return_(
                        instruction_builder.call(
                            instruction_builder.atomic_load(
                                compile_entry_function_pointer(definition, types)?,
                                fmm::ir::AtomicOrdering::Acquire,
                            )?,
                            arguments
                                .iter()
                                .map(|argument| {
                                    fmm::build::variable(argument.name(), argument.type_().clone())
                                })
                                .collect(),
                        )?,
                    ))
                },
            )?;

            Ok(instruction_builder.unreachable())
        },
        fmm::ir::FunctionDefinitionOptions::new()
            .set_calling_convention(fmm::types::CallingConvention::Source)
            .set_linkage(fmm::ir::Linkage::Internal),
    )
}

fn compile_normal_thunk_entry(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &eir::ir::Definition,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    module_builder.define_anonymous_function(
        compile_arguments(definition, types),
        types::compile(definition.result_type(), types),
        |instruction_builder| compile_normal_body(&instruction_builder, definition, types),
        fmm::ir::FunctionDefinitionOptions::new()
            .set_calling_convention(fmm::types::CallingConvention::Source),
    )
}

fn compile_locked_thunk_entry(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &eir::ir::Definition,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let entry_function_name = module_builder.generate_name();

    module_builder.define_function(
        &entry_function_name,
        compile_arguments(definition, types),
        types::compile(definition.result_type(), types),
        |instruction_builder| {
            instruction_builder.if_(
                fmm::build::comparison_operation(
                    fmm::ir::ComparisonOperator::Equal,
                    fmm::build::bit_cast(
                        fmm::types::Primitive::PointerInteger,
                        instruction_builder.atomic_load(
                            compile_entry_function_pointer(definition, types)?,
                            fmm::ir::AtomicOrdering::Acquire,
                        )?,
                    ),
                    fmm::build::bit_cast(
                        fmm::types::Primitive::PointerInteger,
                        fmm::build::variable(
                            &entry_function_name,
                            types::compile_entry_function(definition, types),
                        ),
                    ),
                )?,
                // TODO Return to handle thunk locks asynchronously.
                |instruction_builder| Ok(instruction_builder.unreachable()),
                |instruction_builder| compile_normal_body(&instruction_builder, definition, types),
            )?;

            Ok(instruction_builder.unreachable())
        },
        fmm::ir::FunctionDefinitionOptions::new()
            .set_calling_convention(fmm::types::CallingConvention::Source)
            .set_linkage(fmm::ir::Linkage::Internal),
    )
}

fn compile_normal_body(
    instruction_builder: &fmm::build::InstructionBuilder,
    definition: &eir::ir::Definition,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::ir::Block, CompileError> {
    let value = instruction_builder.load(compile_thunk_value_pointer(definition, types)?)?;

    reference_count::clone_expression(
        instruction_builder,
        &value,
        definition.result_type(),
        types,
    )?;

    Ok(instruction_builder.return_(value))
}

fn compile_entry_function_pointer(
    definition: &eir::ir::Definition,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::bit_cast(
        fmm::types::Pointer::new(types::compile_entry_function(definition, types)),
        closure::compile_entry_function_pointer(compile_closure_pointer(
            definition.type_(),
            types,
        )?)?,
    )
    .into())
}

fn compile_drop_function_pointer(
    definition: &eir::ir::Definition,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    closure::compile_drop_function_pointer(compile_closure_pointer(definition.type_(), types)?)
}

fn compile_arguments(
    definition: &eir::ir::Definition,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Vec<fmm::ir::Argument> {
    vec![fmm::ir::Argument::new(
        CLOSURE_NAME,
        types::compile_untyped_closure_pointer(),
    )]
    .into_iter()
    .chain(definition.arguments().iter().map(|argument| {
        fmm::ir::Argument::new(argument.name(), types::compile(argument.type_(), types))
    }))
    .collect()
}

fn compile_thunk_value_pointer(
    definition: &eir::ir::Definition,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::union_address(compile_payload_pointer(definition, types)?, 1)?.into())
}

fn compile_payload_pointer(
    definition: &eir::ir::Definition,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    closure::compile_environment_pointer(fmm::build::bit_cast(
        fmm::types::Pointer::new(types::compile_sized_closure(definition, types)),
        compile_untyped_closure_pointer(),
    ))
}

fn compile_closure_pointer(
    function_type: &eir::types::Function,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    Ok(fmm::build::bit_cast(
        fmm::types::Pointer::new(types::compile_unsized_closure(function_type, types)),
        compile_untyped_closure_pointer(),
    )
    .into())
}

fn compile_untyped_closure_pointer() -> fmm::build::TypedExpression {
    fmm::build::variable(CLOSURE_NAME, types::compile_untyped_closure_pointer())
}
