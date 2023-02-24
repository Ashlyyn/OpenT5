use std::arch::x86_64::{CpuidResult, __cpuid};

pub fn cpuid(leaf: u32) -> CpuidResult {
    // SAFETY:
    // CPUID should always be safe to execute.
    unsafe { __cpuid(leaf) }
}
