pub fn compile_variant_definition(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &ssf::ir::VariantDefinition,
) -> Result<(), fmm::build::BuildError> {
    // TODO Define GC functions.
    module_builder.define_variable(
        definition.name(),
        fmm::build::record(vec![fmm::ir::Primitive::Integer8(0).into()]),
        false,
        fmm::ir::Linkage::Weak,
    );

    Ok(())
}
