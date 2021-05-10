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
        Expression::ArithmeticOperation(_) => todo!(),
        Expression::Boolean(_) => todo!(),
        Expression::ByteString(_) => todo!(),
        Expression::Case(_) => todo!(),
        Expression::ComparisonOperation(_) => todo!(),
        Expression::FunctionApplication(_) => todo!(),
        Expression::If(_) => todo!(),
        Expression::Let(_) => todo!(),
        Expression::LetRecursive(_) => todo!(),
        Expression::Number(_) => todo!(),
        Expression::Record(_) => todo!(),
        Expression::RecordElement(_) => todo!(),
        Expression::Variable(_) => todo!(),
        Expression::Variant(_) => todo!(),
    }
}
