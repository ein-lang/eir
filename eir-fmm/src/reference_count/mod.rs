mod utilities;

use crate::types;

pub fn clone_variable(
    instruction_builder: &fmm::build::InstructionBuilder,
    variable: &str,
    type_: &eir::types::Type,
) -> Result<(), fmm::build::BuildError> {
    match type_ {
        eir::types::Type::ByteString => {
            let _ = fmm::build::variable(variable, types::compile_string());

            todo!();
        }
        eir::types::Type::Function(_) => todo!(),
        eir::types::Type::Record(_) => todo!(),
        eir::types::Type::Variant => todo!(),
        eir::types::Type::Boolean | eir::types::Type::Number => {}
    }

    Ok(())
}

pub fn drop_variable(
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
