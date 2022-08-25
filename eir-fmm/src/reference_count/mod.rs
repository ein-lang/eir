mod expression;
mod function;
mod heap;
mod pointer;
mod record;
mod record_utilities;
mod variant;

pub use expression::*;
pub use function::*;
pub use heap::*;
pub use pointer::{compile_tagged_pointer, compile_untagged_pointer, drop_pointer};
pub use record::*;
pub use variant::*;

pub(self) fn reference_count_function_definition_options() -> fmm::ir::FunctionDefinitionOptions {
    fmm::ir::FunctionDefinitionOptions::new()
        .set_calling_convention(fmm::types::CallingConvention::Target)
        .set_linkage(fmm::ir::Linkage::Weak)
}
