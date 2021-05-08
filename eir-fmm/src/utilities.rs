pub fn get_environment_from_definition(
    definition: &eir::ir::Definition,
) -> Vec<&eir::ir::Argument> {
    definition
        .environment()
        .iter()
        .filter(|free_variable| free_variable.name() != definition.name())
        .collect()
}
