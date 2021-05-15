use crate::{closures, entry_functions, function_applications, types};
use std::collections::HashMap;

pub fn compile_arity(arity: usize) -> fmm::ir::Primitive {
    fmm::ir::Primitive::PointerInteger(arity as i64)
}

pub fn compile(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    expression: &eir::ir::Expression,
    variables: &HashMap<String, fmm::build::TypedExpression>,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let compile = |expression, variables| {
        compile(
            module_builder,
            instruction_builder,
            expression,
            variables,
            types,
        )
    };

    Ok(match expression {
        eir::ir::Expression::ArithmeticOperation(operation) => compile_arithmetic_operation(
            module_builder,
            instruction_builder,
            operation,
            variables,
            types,
        )?
        .into(),
        eir::ir::Expression::Boolean(boolean) => fmm::ir::Primitive::Boolean(*boolean).into(),
        eir::ir::Expression::Case(case) => {
            compile_case(module_builder, instruction_builder, case, variables, types)?
        }
        eir::ir::Expression::CloneVariables(clone) => {
            for (variable, type_) in clone.variables() {
                clone_variable(instruction_builder, variable, type_);
            }

            compile(clone.expression(), variables)?
        }
        eir::ir::Expression::ComparisonOperation(operation) => compile_comparison_operation(
            module_builder,
            instruction_builder,
            operation,
            variables,
            types,
        )?
        .into(),
        eir::ir::Expression::DropVariables(drop) => {
            for (variable, type_) in drop.variables() {
                drop_variable(instruction_builder, variable, type_);
            }

            compile(drop.expression(), variables)?
        }
        eir::ir::Expression::FunctionApplication(function_application) => {
            function_applications::compile(
                module_builder,
                instruction_builder,
                compile(function_application.first_function(), variables)?,
                &function_application
                    .arguments()
                    .into_iter()
                    .map(|argument| compile(argument, variables))
                    .collect::<Result<Vec<_>, _>>()?,
            )?
        }
        eir::ir::Expression::If(if_) => {
            compile_if(module_builder, instruction_builder, if_, variables, types)?
        }
        eir::ir::Expression::Let(let_) => {
            compile_let(module_builder, instruction_builder, let_, variables, types)?
        }
        eir::ir::Expression::LetRecursive(let_recursive) => compile_let_recursive(
            module_builder,
            instruction_builder,
            let_recursive,
            variables,
            types,
        )?,
        eir::ir::Expression::Number(number) => fmm::ir::Primitive::Float64(*number).into(),
        eir::ir::Expression::Record(record) => {
            let unboxed = fmm::build::record(
                record
                    .elements()
                    .iter()
                    .map(|argument| compile(argument, variables))
                    .collect::<Result<_, _>>()?,
            );

            if types::is_record_boxed(record.type_(), types) {
                let pointer = instruction_builder.allocate_heap(unboxed.type_().clone());

                instruction_builder.store(unboxed, pointer.clone());

                fmm::build::bit_cast(types::compile_record(record.type_(), types), pointer).into()
            } else {
                unboxed.into()
            }
        }
        eir::ir::Expression::RecordElement(element) => {
            let record = compile(element.record(), variables)?;

            instruction_builder.deconstruct_record(
                if types::is_record_boxed(element.type_(), types) {
                    instruction_builder.load(fmm::build::bit_cast(
                        fmm::types::Pointer::new(types::compile_unboxed_record(
                            element.type_(),
                            types,
                        )),
                        record,
                    ))?
                } else {
                    record
                },
                element.index(),
            )?
        }
        eir::ir::Expression::ByteString(string) => fmm::build::record(vec![
            fmm::build::bit_cast(
                fmm::types::Pointer::new(fmm::types::Primitive::Integer8),
                module_builder.define_anonymous_variable(
                    fmm::build::record(
                        string
                            .value()
                            .iter()
                            .map(|&byte| fmm::ir::Primitive::Integer8(byte).into())
                            .collect(),
                    ),
                    false,
                ),
            )
            .into(),
            fmm::ir::Primitive::PointerInteger(string.value().len() as i64).into(),
        ])
        .into(),
        eir::ir::Expression::Variable(variable) => variables[variable.name()].clone(),
        eir::ir::Expression::Variant(variant) => fmm::build::record(vec![
            compile_variant_tag(variant.type_()),
            compile_boxed_payload(
                instruction_builder,
                &compile(variant.payload(), variables)?,
                variant.type_(),
            )?,
        ])
        .into(),
    })
}

fn compile_if(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    if_: &eir::ir::If,
    variables: &HashMap<String, fmm::build::TypedExpression>,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let compile = |instruction_builder: &fmm::build::InstructionBuilder, expression| {
        compile(
            module_builder,
            instruction_builder,
            expression,
            variables,
            types,
        )
    };

    instruction_builder.if_(
        compile(instruction_builder, if_.condition())?,
        |instruction_builder| {
            Ok(instruction_builder.branch(compile(&instruction_builder, if_.then())?))
        },
        |instruction_builder| {
            Ok(instruction_builder.branch(compile(&instruction_builder, if_.else_())?))
        },
    )
}

fn compile_case(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    case: &eir::ir::Case,
    variables: &HashMap<String, fmm::build::TypedExpression>,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let argument = compile(
        module_builder,
        instruction_builder,
        case.argument(),
        variables,
        types,
    )?;

    Ok(compile_alternatives(
        module_builder,
        instruction_builder,
        argument,
        case.alternatives(),
        case.default_alternative(),
        variables,
        types,
    )?
    .unwrap())
}

fn compile_alternatives(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    argument: fmm::build::TypedExpression,
    alternatives: &[eir::ir::Alternative],
    default_alternative: Option<&eir::ir::Expression>,
    variables: &HashMap<String, fmm::build::TypedExpression>,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<Option<fmm::build::TypedExpression>, fmm::build::BuildError> {
    Ok(match alternatives {
        [] => default_alternative
            .map(|default_alternative| {
                compile(
                    module_builder,
                    instruction_builder,
                    default_alternative,
                    variables,
                    types,
                )
            })
            .transpose()?,
        [alternative, ..] => Some(instruction_builder.if_(
            fmm::build::comparison_operation(
                fmm::ir::ComparisonOperator::Equal,
                fmm::build::bit_cast(
                    fmm::types::Primitive::PointerInteger,
                    instruction_builder.deconstruct_record(argument.clone(), 0)?,
                ),
                fmm::build::bit_cast(
                    fmm::types::Primitive::PointerInteger,
                    compile_variant_tag(alternative.type_()),
                ),
            )?,
            |instruction_builder| {
                Ok(instruction_builder.branch(compile(
                    module_builder,
                    &instruction_builder,
                    alternative.expression(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain(vec![(
                            alternative.name().into(),
                            compile_unboxed_payload(
                                &instruction_builder,
                                &instruction_builder.deconstruct_record(argument.clone(), 1)?,
                                alternative.type_(),
                                types,
                            )?,
                        )])
                        .collect(),
                    types,
                )?))
            },
            |instruction_builder| {
                Ok(
                    if let Some(expression) = compile_alternatives(
                        module_builder,
                        &instruction_builder,
                        argument.clone(),
                        &alternatives[1..],
                        default_alternative,
                        variables,
                        types,
                    )? {
                        instruction_builder.branch(expression)
                    } else {
                        instruction_builder.unreachable()
                    },
                )
            },
        )?),
    })
}

fn compile_variant_tag(type_: &eir::types::Type) -> fmm::build::TypedExpression {
    fmm::build::variable(types::compile_type_id(type_), types::compile_variant_tag())
}

fn compile_boxed_payload(
    builder: &fmm::build::InstructionBuilder,
    payload: &fmm::build::TypedExpression,
    variant_type: &eir::types::Type,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    compile_union_bit_cast(
        builder,
        types::compile_variant_payload(),
        // Strings have two words.
        if variant_type == &eir::types::Type::ByteString {
            let pointer = builder.allocate_heap(payload.type_().clone());

            builder.store(payload.clone(), pointer.clone());

            pointer
        } else {
            payload.clone()
        },
    )
}

fn compile_unboxed_payload(
    builder: &fmm::build::InstructionBuilder,
    payload: &fmm::build::TypedExpression,
    variant_type: &eir::types::Type,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    Ok(if variant_type == &eir::types::Type::ByteString {
        builder.load(fmm::build::bit_cast(
            fmm::types::Pointer::new(types::compile(variant_type, types)),
            payload.clone(),
        ))?
    } else {
        compile_union_bit_cast(
            builder,
            types::compile(variant_type, types),
            payload.clone(),
        )?
    })
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

fn compile_let(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    let_: &eir::ir::Let,
    variables: &HashMap<String, fmm::build::TypedExpression>,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let compile = |expression, variables| {
        compile(
            module_builder,
            instruction_builder,
            expression,
            variables,
            types,
        )
    };

    compile(
        let_.expression(),
        &variables
            .clone()
            .drain()
            .chain(vec![(
                let_.name().into(),
                compile(let_.bound_expression(), variables)?,
            )])
            .collect(),
    )
}

fn compile_let_recursive(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    let_: &eir::ir::LetRecursive,
    variables: &HashMap<String, fmm::build::TypedExpression>,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let closure_pointer =
        instruction_builder.allocate_heap(types::compile_sized_closure(let_.definition(), types));

    instruction_builder.store(
        closures::compile_closure_content(
            entry_functions::compile(module_builder, let_.definition(), &variables, types)?,
            let_.definition()
                .environment()
                .iter()
                .map(|free_variable| variables[free_variable.name()].clone())
                .collect(),
        ),
        closure_pointer.clone(),
    );

    compile(
        module_builder,
        instruction_builder,
        let_.expression(),
        &variables
            .clone()
            .into_iter()
            .chain(vec![(
                let_.definition().name().into(),
                fmm::build::bit_cast(
                    fmm::types::Pointer::new(types::compile_unsized_closure(
                        let_.definition().type_(),
                        types,
                    )),
                    closure_pointer,
                )
                .into(),
            )])
            .collect(),
        &types,
    )
}

fn compile_arithmetic_operation(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    operation: &eir::ir::ArithmeticOperation,
    variables: &HashMap<String, fmm::build::TypedExpression>,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::ir::ArithmeticOperation, fmm::build::BuildError> {
    let compile = |expression| {
        compile(
            module_builder,
            instruction_builder,
            expression,
            variables,
            types,
        )
    };

    let lhs = compile(operation.lhs())?;
    let rhs = compile(operation.rhs())?;

    Ok(match operation.operator() {
        eir::ir::ArithmeticOperator::Add => {
            fmm::build::arithmetic_operation(fmm::ir::ArithmeticOperator::Add, lhs, rhs)?
        }
        eir::ir::ArithmeticOperator::Subtract => {
            fmm::build::arithmetic_operation(fmm::ir::ArithmeticOperator::Subtract, lhs, rhs)?
        }
        eir::ir::ArithmeticOperator::Multiply => {
            fmm::build::arithmetic_operation(fmm::ir::ArithmeticOperator::Multiply, lhs, rhs)?
        }
        eir::ir::ArithmeticOperator::Divide => {
            fmm::build::arithmetic_operation(fmm::ir::ArithmeticOperator::Divide, lhs, rhs)?
        }
    })
}

fn compile_comparison_operation(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    operation: &eir::ir::ComparisonOperation,
    variables: &HashMap<String, fmm::build::TypedExpression>,
    types: &HashMap<String, eir::types::RecordBody>,
) -> Result<fmm::ir::ComparisonOperation, fmm::build::BuildError> {
    let compile = |expression| {
        compile(
            module_builder,
            instruction_builder,
            expression,
            variables,
            types,
        )
    };

    let lhs = compile(operation.lhs())?;
    let rhs = compile(operation.rhs())?;

    fmm::build::comparison_operation(
        match operation.operator() {
            eir::ir::ComparisonOperator::Equal => fmm::ir::ComparisonOperator::Equal,
            eir::ir::ComparisonOperator::NotEqual => fmm::ir::ComparisonOperator::NotEqual,
            eir::ir::ComparisonOperator::GreaterThan => fmm::ir::ComparisonOperator::GreaterThan,
            eir::ir::ComparisonOperator::GreaterThanOrEqual => {
                fmm::ir::ComparisonOperator::GreaterThanOrEqual
            }
            eir::ir::ComparisonOperator::LessThan => fmm::ir::ComparisonOperator::LessThan,
            eir::ir::ComparisonOperator::LessThanOrEqual => {
                fmm::ir::ComparisonOperator::LessThanOrEqual
            }
        },
        lhs,
        rhs,
    )
}

fn clone_variable(
    instruction_builder: &fmm::build::InstructionBuilder,
    variable: &str,
    type_: &eir::types::Type,
) -> Result<(), fmm::build::BuildError> {
    match type_ {
        eir::types::Type::ByteString => todo!(),
        eir::types::Type::Function(_) => todo!(),
        eir::types::Type::Record(_) => todo!(),
        eir::types::Type::Variant => todo!(),
        eir::types::Type::Boolean | eir::types::Type::Number => {}
    }

    Ok(())
}

fn drop_variable(
    instruction_builder: &fmm::build::InstructionBuilder,
    variable: &str,
    type_: &eir::types::Type,
) -> Result<(), fmm::build::BuildError> {
    match type_ {
        eir::types::Type::ByteString => todo!(),
        eir::types::Type::Function(_) => todo!(),
        eir::types::Type::Record(_) => todo!(),
        eir::types::Type::Variant => todo!(),
        eir::types::Type::Boolean | eir::types::Type::Number => {}
    }

    Ok(())
}
