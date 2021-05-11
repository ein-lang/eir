mod error;

use crate::ir::*;
pub use error::ReferenceCountError;
use std::collections::HashSet;

pub fn count_references(module: &Module) -> Result<Module, ReferenceCountError> {
    Ok(Module::new(
        module.type_definitions().to_vec(),
        module.foreign_declarations().to_vec(),
        module.foreign_definitions().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(convert_definition)
            .collect::<Result<_, _>>()?,
    ))
}

fn convert_definition(definition: &Definition) -> Result<Definition, ReferenceCountError> {
    // TODO Do not depend on the Definition::environment() API.
    let (expression, _) = convert_expression(
        definition.body(),
        &vec![definition.name().into()]
            .into_iter()
            .chain(
                definition
                    .environment()
                    .iter()
                    .chain(definition.arguments())
                    .map(|argument| argument.name().into()),
            )
            .collect(),
        &Default::default(),
    )?;

    Ok(Definition::with_options(
        definition.name(),
        definition.environment().to_vec(),
        definition.arguments().to_vec(),
        expression,
        definition.result_type().clone(),
        definition.is_thunk(),
    ))
}

fn convert_expression(
    expression: &Expression,
    owned_variables: &HashSet<String>,
    moved_variables: &HashSet<String>,
) -> Result<(Expression, HashSet<String>), ReferenceCountError> {
    Ok(match expression {
        Expression::ArithmeticOperation(operation) => {
            let (rhs, moved_variables) =
                convert_expression(operation.rhs(), owned_variables, &moved_variables)?;
            let (lhs, moved_variables) =
                convert_expression(operation.lhs(), owned_variables, &moved_variables)?;

            (
                ArithmeticOperation::new(operation.operator(), lhs, rhs).into(),
                moved_variables,
            )
        }
        Expression::Case(case) => {
            let (default_alternative, default_alternative_moved_variables) =
                if let Some(expression) = case.default_alternative() {
                    let (expression, moved_variables) =
                        convert_expression(expression, owned_variables, moved_variables)?;

                    (Some(expression), moved_variables)
                } else {
                    (None, Default::default())
                };

            let alternative_tuples = case
                .alternatives()
                .iter()
                .map(|alternative| {
                    let (expression, moved_variables) = convert_expression(
                        alternative.expression(),
                        &owned_variables
                            .into_iter()
                            .cloned()
                            .chain(vec![alternative.name().into()])
                            .collect(),
                        moved_variables,
                    )?;

                    Ok((
                        Alternative::new(
                            alternative.type_().clone(),
                            alternative.name(),
                            expression,
                        ),
                        moved_variables,
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let all_moved_variables = default_alternative_moved_variables
                .clone()
                .into_iter()
                .chain(
                    alternative_tuples
                        .iter()
                        .flat_map(|(_, moved_variables)| moved_variables.clone()),
                )
                .collect();

            let (argument, moved_variables) =
                convert_expression(case.argument(), owned_variables, &all_moved_variables)?;

            (
                Case::new(
                    argument,
                    alternative_tuples
                        .into_iter()
                        .map(|(alternative, moved_variables)| {
                            Alternative::new(
                                alternative.type_().clone(),
                                alternative.name(),
                                drop_variables(
                                    alternative.expression(),
                                    &all_moved_variables
                                        .difference(&moved_variables)
                                        .cloned()
                                        .collect(),
                                ),
                            )
                        })
                        .collect(),
                    default_alternative.map(|expression| {
                        drop_variables(
                            &expression,
                            &all_moved_variables
                                .difference(&default_alternative_moved_variables)
                                .cloned()
                                .collect(),
                        )
                    }),
                )
                .into(),
                moved_variables,
            )
        }
        Expression::ComparisonOperation(operation) => {
            let (rhs, moved_variables) =
                convert_expression(operation.rhs(), owned_variables, &moved_variables)?;
            let (lhs, moved_variables) =
                convert_expression(operation.lhs(), owned_variables, &moved_variables)?;

            (
                ComparisonOperation::new(operation.operator(), lhs, rhs).into(),
                moved_variables,
            )
        }
        Expression::FunctionApplication(application) => {
            let (argument, moved_variables) =
                convert_expression(application.function(), owned_variables, moved_variables)?;
            let (function, moved_variables) =
                convert_expression(application.function(), owned_variables, &moved_variables)?;

            (
                FunctionApplication::new(function, argument).into(),
                moved_variables,
            )
        }
        Expression::If(if_) => {
            let (then, then_moved_variables) =
                convert_expression(if_.then(), owned_variables, moved_variables)?;
            let (else_, else_moved_variables) =
                convert_expression(if_.else_(), owned_variables, moved_variables)?;

            let all_moved_variables = then_moved_variables
                .clone()
                .into_iter()
                .chain(else_moved_variables.clone())
                .collect();

            let (condition, moved_variables) =
                convert_expression(if_.condition(), owned_variables, &all_moved_variables)?;

            (
                If::new(
                    condition,
                    drop_variables(
                        &then,
                        &all_moved_variables
                            .difference(&then_moved_variables)
                            .cloned()
                            .collect(),
                    ),
                    drop_variables(
                        &else_,
                        &all_moved_variables
                            .difference(&else_moved_variables)
                            .cloned()
                            .collect(),
                    ),
                )
                .into(),
                moved_variables,
            )
        }
        Expression::Let(let_) => {
            let (expression, moved_variables) = convert_expression(
                let_.expression(),
                &owned_variables
                    .iter()
                    .cloned()
                    .chain(vec![let_.name().into()])
                    .collect(),
                moved_variables,
            )?;
            let (bound_expression, moved_variables) =
                convert_expression(let_.bound_expression(), owned_variables, &moved_variables)?;

            (
                Let::new(
                    let_.name(),
                    let_.type_().clone(),
                    bound_expression,
                    expression,
                )
                .into(),
                moved_variables,
            )
        }
        Expression::LetRecursive(let_) => {
            let (expression, moved_variables) = convert_expression(
                let_.expression(),
                &owned_variables
                    .iter()
                    .cloned()
                    .chain(vec![let_.definition().name().into()])
                    .collect(),
                moved_variables,
            )?;

            (
                LetRecursive::new(convert_definition(let_.definition())?, expression).into(),
                moved_variables,
            )
        }
        Expression::Record(record) => {
            let (elements, moved_variables) = record.elements().iter().rev().fold(
                Ok((vec![], moved_variables.clone())),
                |result, element| {
                    let (elements, moved_variables) = result?;
                    let (element, moved_variables) =
                        convert_expression(element, owned_variables, &moved_variables)?;

                    Ok((
                        vec![element].into_iter().chain(elements).collect(),
                        moved_variables,
                    ))
                },
            )?;

            (
                Record::new(record.type_().clone(), elements).into(),
                moved_variables,
            )
        }
        Expression::RecordElement(element) => {
            let (record, moved_variables) =
                convert_expression(element.record(), owned_variables, moved_variables)?;

            (
                RecordElement::new(element.type_().clone(), element.index(), record).into(),
                moved_variables,
            )
        }
        Expression::Variable(variable) => {
            if owned_variables.contains(variable.name())
                && moved_variables.contains(variable.name())
            {
                (
                    CloneVariable::new(variable.clone()).into(),
                    moved_variables.clone(),
                )
            } else {
                (
                    variable.clone().into(),
                    moved_variables
                        .clone()
                        .into_iter()
                        .chain(vec![variable.name().into()])
                        .collect(),
                )
            }
        }
        Expression::Variant(variant) => {
            let (expression, moved_variables) =
                convert_expression(variant.payload(), owned_variables, moved_variables)?;

            (
                Variant::new(variant.type_().clone(), expression).into(),
                moved_variables,
            )
        }
        Expression::Boolean(_) | Expression::ByteString(_) | Expression::Number(_) => {
            (expression.clone(), moved_variables.clone())
        }
        Expression::CloneVariable(_) | Expression::DropVariable(_) => {
            return Err(ReferenceCountError::ExpressionNotSupported(
                expression.clone(),
            ));
        }
    })
}

fn drop_variables(expression: &Expression, dropped_variables: &HashSet<String>) -> Expression {
    dropped_variables
        .iter()
        .fold(expression.clone(), |expression, variable| {
            DropVariable::new(Variable::new(variable), expression).into()
        })
}
