mod expressions;
mod functions;
mod heap;
mod pointers;
mod record_utilities;
mod records;
mod variants;

pub use expressions::*;
pub use functions::*;
pub use heap::*;
pub use pointers::{compile_tagged_pointer, compile_untagged_pointer, drop_pointer};
pub use records::*;
pub use variants::*;

pub(self) fn reference_count_function_definition_options() -> fmm::ir::FunctionDefinitionOptions {
    return fmm::ir::FunctionDefinitionOptions::new()
        .set_calling_convention(fmm::types::CallingConvention::Target)
        .set_linkage(fmm::ir::Linkage::Weak);
}
