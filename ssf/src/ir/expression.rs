use super::arithmetic_operation::ArithmeticOperation;
use super::bit_cast::BitCast;
use super::case::Case;
use super::comparison_operation::ComparisonOperation;
use super::function_application::FunctionApplication;
use super::let_::Let;
use super::let_recursive::LetRecursive;
use super::primitive::Primitive;
use super::primitive_case::PrimitiveCase;
use super::record::Record;
use super::variable::Variable;
use super::variant::Variant;
use super::variant_case::VariantCase;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    ArithmeticOperation(ArithmeticOperation),
    BitCast(BitCast),
    Case(Case),
    ComparisonOperation(ComparisonOperation),
    FunctionApplication(FunctionApplication),
    Let(Let),
    LetRecursive(LetRecursive),
    Primitive(Primitive),
    Record(Record),
    Variable(Variable),
    Variant(Variant),
}

impl Expression {
    pub fn to_variable(&self) -> Option<&Variable> {
        match self {
            Self::Variable(variable) => Some(variable),
            _ => None,
        }
    }

    pub(crate) fn find_variables(&self) -> HashSet<String> {
        match self {
            Self::ArithmeticOperation(operation) => operation.find_variables(),
            Self::BitCast(bit_cast) => bit_cast.find_variables(),
            Self::Case(case) => case.find_variables(),
            Self::ComparisonOperation(operation) => operation.find_variables(),
            Self::Record(record) => record.find_variables(),
            Self::FunctionApplication(function_application) => {
                function_application.find_variables()
            }
            Self::LetRecursive(let_recursive) => let_recursive.find_variables(),
            Self::Let(let_) => let_.find_variables(),
            Self::Variable(variable) => variable.find_variables(),
            Self::Variant(variant) => variant.find_variables(),
            Self::Primitive(_) => HashSet::new(),
        }
    }

    pub(crate) fn infer_environment(&self, variables: &HashMap<String, Type>) -> Self {
        match self {
            Self::ArithmeticOperation(operation) => operation.infer_environment(variables).into(),
            Self::BitCast(bit_cast) => bit_cast.infer_environment(variables).into(),
            Self::Case(case) => case.infer_environment(variables).into(),
            Self::ComparisonOperation(operation) => operation.infer_environment(variables).into(),
            Self::Record(record) => {
                record.infer_environment(variables).into()
            }
            Self::FunctionApplication(function_application) => {
                function_application.infer_environment(variables).into()
            }
            Self::LetRecursive(let_recursive) => let_recursive.infer_environment(variables).into(),
            Self::Let(let_) => let_.infer_environment(variables).into(),
            Self::Variant(variant) => variant.infer_environment(variables).into(),
            Self::Primitive(_) | Self::Variable(_) => self.clone(),
        }
    }

    pub(crate) fn convert_types(&self, convert: &impl Fn(&Type) -> Type) -> Self {
        match self {
            Self::ArithmeticOperation(operation) => operation.convert_types(convert).into(),
            Self::BitCast(bit_cast) => bit_cast.convert_types(convert).into(),
            Self::Case(case) => case.convert_types(convert).into(),
            Self::ComparisonOperation(operation) => operation.convert_types(convert).into(),
            Self::Record(record) => {
                record.convert_types(convert).into()
            }
            Self::FunctionApplication(function_application) => {
                function_application.convert_types(convert).into()
            }
            Self::LetRecursive(let_recursive) => let_recursive.convert_types(convert).into(),
            Self::Let(let_) => let_.convert_types(convert).into(),
            Self::Variant(variant) => variant.convert_types(convert).into(),
            Self::Primitive(_) | Self::Variable(_) => self.clone(),
        }
    }
}

impl From<VariantCase> for Expression {
    fn from(variant_case: VariantCase) -> Self {
        Self::Case(variant_case.into())
    }
}

impl From<ArithmeticOperation> for Expression {
    fn from(operation: ArithmeticOperation) -> Self {
        Self::ArithmeticOperation(operation)
    }
}

impl From<BitCast> for Expression {
    fn from(bit_cast: BitCast) -> Self {
        Self::BitCast(bit_cast)
    }
}

impl From<Case> for Expression {
    fn from(case: Case) -> Self {
        Self::Case(case)
    }
}

impl From<ComparisonOperation> for Expression {
    fn from(operation: ComparisonOperation) -> Self {
        Self::ComparisonOperation(operation)
    }
}

impl From<Record> for Expression {
    fn from(record: Record) -> Self {
        Self::Record(record)
    }
}

impl From<FunctionApplication> for Expression {
    fn from(function_application: FunctionApplication) -> Self {
        Self::FunctionApplication(function_application)
    }
}

impl From<LetRecursive> for Expression {
    fn from(let_recursive: LetRecursive) -> Self {
        Self::LetRecursive(let_recursive)
    }
}

impl From<Let> for Expression {
    fn from(let_: Let) -> Self {
        Self::Let(let_)
    }
}

impl<T: Into<Primitive>> From<T> for Expression {
    fn from(primitive: T) -> Self {
        Self::Primitive(primitive.into())
    }
}

impl From<PrimitiveCase> for Expression {
    fn from(primitive_case: PrimitiveCase) -> Self {
        Self::Case(primitive_case.into())
    }
}

impl From<Variable> for Expression {
    fn from(variable: Variable) -> Self {
        Self::Variable(variable)
    }
}

impl From<Variant> for Expression {
    fn from(variant: Variant) -> Self {
        Self::Variant(variant)
    }
}
