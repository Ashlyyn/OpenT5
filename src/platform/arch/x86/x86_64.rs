use std::arch::x86_64::{__cpuid, CpuidResult};

pub fn cpuid(leaf: u32) -> CpuidResult {
    unsafe { __cpuid(leaf) }
}