use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    FmmBuild(fmm::build::BuildError),
    NestedVariant,
    ReferenceCount(eir::analysis::ReferenceCountError),
    TypeCheck(eir::analysis::TypeCheckError),
}

impl Display for CompileError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

impl Error for CompileError {}

impl From<fmm::build::BuildError> for CompileError {
    fn from(error: fmm::build::BuildError) -> Self {
        Self::FmmBuild(error)
    }
}

impl From<eir::analysis::ReferenceCountError> for CompileError {
    fn from(error: eir::analysis::ReferenceCountError) -> Self {
        Self::ReferenceCount(error)
    }
}

impl From<eir::analysis::TypeCheckError> for CompileError {
    fn from(error: eir::analysis::TypeCheckError) -> Self {
        Self::TypeCheck(error)
    }
}
