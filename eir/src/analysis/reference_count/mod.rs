use crate::ir::*;

pub fn count_references(module: &Module) -> Module {
    Module::new(
        module.type_definitions().to_vec(),
        module.foreign_declarations().to_vec(),
        module.foreign_definitions().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(convert_definition)
            .collect(),
    )
}

fn convert_definition(definition: &Definition) -> Definition {
    Definition::with_options(
        definition.name(),
        definition.environment().to_vec(),
        definition.arguments().to_vec(),
        convert_expression(definition.body()),
        definition.result_type().clone(),
        definition.is_thunk(),
    )
}

fn convert_expression(expression: &Expression) -> Expression {
    match expression {
        Expression::ArithmeticOperation(operation) => ArithmeticOperation::new(
            operation.operator(),
            convert_expression(operation.lhs()),
            convert_expression(operation.rhs()),
        )
        .into(),
        Expression::Case(case) => Case::new(
            convert_expression(case.argument()),
            case.alternatives()
                .iter()
                .map(|alternative| {
                    Alternative::new(
                        alternative.type_().clone(),
                        alternative.name(),
                        convert_expression(alternative.expression()),
                    )
                })
                .collect(),
            case.default_alternative().map(convert_expression),
        )
        .into(),
        Expression::ComparisonOperation(operation) => ComparisonOperation::new(
            operation.operator(),
            convert_expression(operation.lhs()),
            convert_expression(operation.rhs()),
        )
        .into(),
        Expression::FunctionApplication(application) => FunctionApplication::new(
            convert_expression(application.function()),
            convert_expression(application.argument()),
        )
        .into(),
        Expression::If(if_) => If::new(
            convert_expression(if_.condition()),
            convert_expression(if_.then()),
            convert_expression(if_.else_()),
        )
        .into(),
        Expression::Let(let_) => Let::new(
            let_.name(),
            let_.type_().clone(),
            convert_expression(let_.bound_expression()),
            convert_expression(let_.expression()),
        )
        .into(),
        Expression::LetRecursive(let_) => LetRecursive::new(
            convert_definition(let_.definition()),
            convert_expression(let_.expression()),
        )
        .into(),
        Expression::Record(record) => Record::new(
            record.type_().clone(),
            record.elements().iter().map(convert_expression).collect(),
        )
        .into(),
        Expression::RecordElement(element) => RecordElement::new(
            element.type_().clone(),
            element.index(),
            convert_expression(element.record()),
        )
        .into(),
        Expression::Variable(variable) => variable.clone().into(),
        Expression::Variant(variant) => Variant::new(
            variant.type_().clone(),
            convert_expression(variant.payload()),
        )
        .into(),
        Expression::Boolean(_) | Expression::ByteString(_) | Expression::Number(_) => {
            expression.clone()
        }
    }
}
