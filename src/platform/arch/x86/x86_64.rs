use std::arch::x86_64::{CpuidResult, __cpuid};

pub fn cpuid(leaf: u32) -> CpuidResult {
    unsafe { __cpuid(leaf) }
}
