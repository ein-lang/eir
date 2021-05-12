mod error;

use crate::ir::*;
pub use error::ReferenceCountError;
use std::collections::HashSet;

// Closure environments need to be inferred before reference counting.
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
    let owned_variables = vec![definition.name().into()]
        .into_iter()
        .chain(
            definition
                .environment()
                .iter()
                .chain(definition.arguments())
                .map(|argument| argument.name().into()),
        )
        .collect();

    let (expression, moved_variables) =
        convert_expression(definition.body(), &owned_variables, &Default::default())?;

    Ok(Definition::with_options(
        definition.name(),
        definition.environment().to_vec(),
        definition.arguments().to_vec(),
        drop_variables(
            expression,
            owned_variables
                .difference(&moved_variables)
                .cloned()
                .collect(),
        ),
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
                    (None, moved_variables.clone())
                };

            let alternative_tuples = case
                .alternatives()
                .iter()
                .map(|alternative| {
                    let (expression, moved_variables) = convert_expression(
                        alternative.expression(),
                        &owned_variables
                            .iter()
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
                                    alternative.expression().clone(),
                                    all_moved_variables
                                        .difference(&moved_variables)
                                        .cloned()
                                        .collect(),
                                ),
                            )
                        })
                        .collect(),
                    default_alternative.map(|expression| {
                        drop_variables(
                            expression,
                            all_moved_variables
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
                convert_expression(application.argument(), owned_variables, moved_variables)?;
            // TODO Borrow functions.
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
                        then,
                        all_moved_variables
                            .difference(&then_moved_variables)
                            .cloned()
                            .collect(),
                    ),
                    drop_variables(
                        else_,
                        all_moved_variables
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
            let (expression, expression_moved_variables) = convert_expression(
                let_.expression(),
                &owned_variables
                    .iter()
                    .cloned()
                    .chain(vec![let_.name().into()])
                    .collect(),
                moved_variables,
            )?;
            let (bound_expression, moved_variables) = convert_expression(
                let_.bound_expression(),
                owned_variables,
                &expression_moved_variables
                    .iter()
                    .cloned()
                    .filter(|variable| variable != let_.name())
                    .collect(),
            )?;

            (
                Let::new(
                    let_.name(),
                    let_.type_().clone(),
                    bound_expression,
                    if expression_moved_variables.contains(let_.name()) {
                        expression
                    } else {
                        drop_variables(expression, vec![let_.name().into()].into_iter().collect())
                    },
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
                &moved_variables,
            )?;
            let cloned_variables = let_
                .definition()
                .environment()
                .iter()
                .filter_map(|argument| {
                    if should_clone_variable(argument.name(), owned_variables, &moved_variables) {
                        Some(argument.name().into())
                    } else {
                        None
                    }
                })
                .collect::<HashSet<_>>();

            let let_ = LetRecursive::new(
                convert_definition(let_.definition())?,
                if moved_variables.contains(let_.definition().name()) {
                    expression
                } else {
                    drop_variables(
                        expression,
                        vec![let_.definition().name().into()].into_iter().collect(),
                    )
                },
            );

            let moved_variables = moved_variables
                .into_iter()
                .filter(|variable| variable != let_.definition().name())
                .chain(
                    let_.definition()
                        .environment()
                        .iter()
                        .map(|argument| argument.name().into()),
                )
                .collect::<HashSet<String>>();

            (clone_variables(let_, cloned_variables), moved_variables)
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
            if should_clone_variable(variable.name(), owned_variables, moved_variables) {
                (
                    clone_variables(
                        variable.clone(),
                        vec![variable.name().into()].into_iter().collect(),
                    ),
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
        Expression::CloneVariables(_) | Expression::DropVariables(_) => {
            return Err(ReferenceCountError::ExpressionNotSupported(
                expression.clone(),
            ));
        }
    })
}

fn clone_variables(
    expression: impl Into<Expression>,
    cloned_variables: HashSet<String>,
) -> Expression {
    let expression = expression.into();

    if cloned_variables.is_empty() {
        expression
    } else {
        CloneVariables::new(cloned_variables, expression).into()
    }
}

fn drop_variables(
    expression: impl Into<Expression>,
    dropped_variables: HashSet<String>,
) -> Expression {
    let expression = expression.into();

    if dropped_variables.is_empty() {
        expression
    } else {
        DropVariables::new(dropped_variables, expression).into()
    }
}

fn should_clone_variable(
    variable: &str,
    owned_variables: &HashSet<String>,
    moved_variables: &HashSet<String>,
) -> bool {
    owned_variables.contains(variable) && moved_variables.contains(variable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{self, Type};

    #[test]
    fn convert_record() {
        assert_eq!(
            convert_expression(
                &Record::new(
                    types::Record::new("a"),
                    vec![Variable::new("x").into(), Variable::new("x").into()]
                )
                .into(),
                &vec!["x".into()].into_iter().collect(),
                &Default::default()
            )
            .unwrap(),
            (
                Record::new(
                    types::Record::new("a"),
                    vec![
                        CloneVariables::new(
                            vec!["x".into()].into_iter().collect(),
                            Variable::new("x")
                        )
                        .into(),
                        Variable::new("x").into()
                    ]
                )
                .into(),
                vec!["x".into()].into_iter().collect()
            ),
        );
    }

    mod function_applications {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn convert_single() {
            assert_eq!(
                convert_expression(
                    &FunctionApplication::new(Variable::new("f"), Variable::new("x")).into(),
                    &vec!["f".into(), "x".into()].into_iter().collect(),
                    &vec!["f".into(), "x".into()].into_iter().collect(),
                )
                .unwrap(),
                (
                    FunctionApplication::new(
                        CloneVariables::new(
                            vec!["f".into()].into_iter().collect(),
                            Variable::new("f")
                        ),
                        CloneVariables::new(
                            vec!["x".into()].into_iter().collect(),
                            Variable::new("x")
                        )
                    )
                    .into(),
                    vec!["f".into(), "x".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn convert_multiple() {
            assert_eq!(
                convert_expression(
                    &FunctionApplication::new(
                        FunctionApplication::new(Variable::new("f"), Variable::new("x")),
                        Variable::new("x")
                    )
                    .into(),
                    &vec!["f".into(), "x".into()].into_iter().collect(),
                    &Default::default(),
                )
                .unwrap(),
                (
                    FunctionApplication::new(
                        FunctionApplication::new(
                            Variable::new("f"),
                            CloneVariables::new(
                                vec!["x".into()].into_iter().collect(),
                                Variable::new("x")
                            )
                        ),
                        Variable::new("x")
                    )
                    .into(),
                    vec!["f".into(), "x".into()].into_iter().collect()
                ),
            );
        }
    }

    mod let_ {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn convert_with_moved_variable() {
            assert_eq!(
                convert_expression(
                    &Let::new("x", Type::Number, 42.0, Variable::new("x")).into(),
                    &Default::default(),
                    &Default::default()
                )
                .unwrap()
                .0,
                Let::new("x", Type::Number, 42.0, Variable::new("x")).into(),
            );
        }

        #[test]
        fn convert_with_cloned_variable() {
            assert_eq!(
                convert_expression(
                    &Let::new(
                        "x",
                        Type::Number,
                        42.0,
                        ArithmeticOperation::new(
                            ArithmeticOperator::Add,
                            Variable::new("x"),
                            Variable::new("x")
                        ),
                    )
                    .into(),
                    &Default::default(),
                    &Default::default()
                )
                .unwrap()
                .0,
                Let::new(
                    "x",
                    Type::Number,
                    42.0,
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        CloneVariables::new(
                            vec!["x".into()].into_iter().collect(),
                            Variable::new("x")
                        ),
                        Variable::new("x")
                    ),
                )
                .into(),
            );
        }

        #[test]
        fn convert_with_dropped_variable() {
            assert_eq!(
                convert_expression(
                    &Let::new("x", Type::Number, 42.0, 42.0,).into(),
                    &Default::default(),
                    &Default::default()
                )
                .unwrap()
                .0,
                Let::new(
                    "x",
                    Type::Number,
                    42.0,
                    DropVariables::new(vec!["x".into()].into_iter().collect(), 42.0)
                )
                .into(),
            );
        }

        #[test]
        fn convert_with_moved_variable_in_bound_expression() {
            assert_eq!(
                convert_expression(
                    &Let::new("x", Type::Number, Variable::new("y"), Variable::new("x")).into(),
                    &vec!["y".into()].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    Let::new("x", Type::Number, Variable::new("y"), Variable::new("x")).into(),
                    vec!["y".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn convert_with_cloned_variable_in_bound_expression() {
            assert_eq!(
                convert_expression(
                    &Let::new("x", Type::Number, Variable::new("y"), Variable::new("y")).into(),
                    &vec!["y".into()].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    Let::new(
                        "x",
                        Type::Number,
                        CloneVariables::new(
                            vec!["y".into()].into_iter().collect(),
                            Variable::new("y")
                        ),
                        DropVariables::new(
                            vec!["x".into()].into_iter().collect(),
                            Variable::new("y")
                        )
                    )
                    .into(),
                    vec!["y".into()].into_iter().collect()
                ),
            );
        }
    }

    mod let_recursive {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn convert_with_moved_variable() {
            assert_eq!(
                convert_expression(
                    &LetRecursive::new(
                        Definition::new(
                            "f",
                            vec![Argument::new("x", Type::Number)],
                            42.0,
                            Type::Number
                        ),
                        Variable::new("f")
                    )
                    .into(),
                    &Default::default(),
                    &Default::default()
                )
                .unwrap()
                .0,
                LetRecursive::new(
                    Definition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        DropVariables::new(
                            vec!["f".into(), "x".into()].into_iter().collect(),
                            42.0,
                        ),
                        Type::Number
                    ),
                    Variable::new("f")
                )
                .into(),
            );
        }

        #[test]
        fn convert_with_cloned_variable() {
            assert_eq!(
                convert_expression(
                    &LetRecursive::new(
                        Definition::new(
                            "f",
                            vec![Argument::new("x", Type::Number)],
                            42.0,
                            Type::Number
                        ),
                        FunctionApplication::new(
                            FunctionApplication::new(Variable::new("g"), Variable::new("f")),
                            Variable::new("f")
                        )
                    )
                    .into(),
                    &Default::default(),
                    &Default::default()
                )
                .unwrap()
                .0,
                LetRecursive::new(
                    Definition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        DropVariables::new(
                            vec!["f".into(), "x".into()].into_iter().collect(),
                            42.0,
                        ),
                        Type::Number
                    ),
                    FunctionApplication::new(
                        FunctionApplication::new(
                            Variable::new("g"),
                            CloneVariables::new(
                                vec!["f".into()].into_iter().collect(),
                                Variable::new("f")
                            )
                        ),
                        Variable::new("f")
                    )
                )
                .into(),
            );
        }

        #[test]
        fn convert_with_dropped_variable() {
            assert_eq!(
                convert_expression(
                    &LetRecursive::new(
                        Definition::new(
                            "f",
                            vec![Argument::new("x", Type::Number)],
                            42.0,
                            Type::Number
                        ),
                        42.0,
                    )
                    .into(),
                    &Default::default(),
                    &Default::default()
                )
                .unwrap()
                .0,
                LetRecursive::new(
                    Definition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        DropVariables::new(
                            vec!["f".into(), "x".into()].into_iter().collect(),
                            42.0,
                        ),
                        Type::Number
                    ),
                    DropVariables::new(vec!["f".into()].into_iter().collect(), 42.0,)
                )
                .into(),
            );
        }

        #[test]
        fn convert_with_moved_variable_in_environment() {
            assert_eq!(
                convert_expression(
                    &LetRecursive::new(
                        Definition::with_environment(
                            "f",
                            vec![Argument::new("y", Type::Number)],
                            vec![Argument::new("x", Type::Number)],
                            42.0,
                            Type::Number
                        ),
                        Variable::new("f")
                    )
                    .into(),
                    &vec!["y".into()].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    LetRecursive::new(
                        Definition::with_environment(
                            "f",
                            vec![Argument::new("y", Type::Number)],
                            vec![Argument::new("x", Type::Number)],
                            DropVariables::new(
                                vec!["f".into(), "x".into(), "y".into()]
                                    .into_iter()
                                    .collect(),
                                42.0,
                            ),
                            Type::Number
                        ),
                        Variable::new("f")
                    )
                    .into(),
                    vec!["y".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn convert_with_cloned_variable_in_environment() {
            assert_eq!(
                convert_expression(
                    &LetRecursive::new(
                        Definition::with_environment(
                            "f",
                            vec![Argument::new("y", Type::Number)],
                            vec![Argument::new("x", Type::Number)],
                            42.0,
                            Type::Number
                        ),
                        FunctionApplication::new(Variable::new("f"), Variable::new("y"))
                    )
                    .into(),
                    &vec!["y".into()].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    CloneVariables::new(
                        vec!["y".into()].into_iter().collect(),
                        LetRecursive::new(
                            Definition::with_environment(
                                "f",
                                vec![Argument::new("y", Type::Number)],
                                vec![Argument::new("x", Type::Number)],
                                DropVariables::new(
                                    vec!["f".into(), "x".into(), "y".into()]
                                        .into_iter()
                                        .collect(),
                                    42.0,
                                ),
                                Type::Number
                            ),
                            FunctionApplication::new(Variable::new("f"), Variable::new("y"))
                        )
                    )
                    .into(),
                    vec!["y".into()].into_iter().collect()
                ),
            );
        }
    }

    mod definitions {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn convert_with_dropped_argument() {
            assert_eq!(
                convert_definition(&Definition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    42.0,
                    Type::Number
                ))
                .unwrap(),
                Definition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    DropVariables::new(vec!["f".into(), "x".into()].into_iter().collect(), 42.0),
                    Type::Number
                ),
            );
        }

        #[test]
        fn convert_with_dropped_free_variable() {
            assert_eq!(
                convert_definition(&Definition::with_environment(
                    "f",
                    vec![Argument::new("y", Type::Number)],
                    vec![Argument::new("x", Type::Number)],
                    42.0,
                    Type::Number
                ))
                .unwrap(),
                Definition::with_environment(
                    "f",
                    vec![Argument::new("y", Type::Number)],
                    vec![Argument::new("x", Type::Number)],
                    DropVariables::new(
                        vec!["f".into(), "x".into(), "y".into()]
                            .into_iter()
                            .collect(),
                        42.0
                    ),
                    Type::Number
                ),
            );
        }
    }
}
