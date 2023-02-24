use std::arch::x86::{CpuidResult, __cpuid};

pub fn cpuid(leaf: u32) -> CpuidResult {
    unsafe { __cpuid(leaf) }
}
