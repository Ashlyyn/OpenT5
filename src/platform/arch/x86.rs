#![allow(clippy::pub_use)]
use core::{ffi::CStr, mem::transmute};

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_arch = "x86")] {
        pub mod i686;
        pub use i686 as target;
    } else if #[cfg(target_arch = "x86_64")] {
        pub mod x86_64;
        pub use x86_64 as target;
    }
}

pub fn detect_cpu_vendor_and_name() -> (Option<String>, Option<String>) {
    let res = target::cpuid(0x0000_0000);
    let mut vendor_buf =
        // SAFETY:
        // transmute is just changing a [u32; 3] to [u8; 12]. The size remains the
        // same, no invalid types can be created, so it should always be safe.
        unsafe { transmute::<_, [u8; 12]>([res.ebx, res.ecx, res.edx]) }
            .to_vec();
    vendor_buf.push(b'\0');
    let vendor = CStr::from_bytes_until_nul(&vendor_buf).map_or(None, |s| {
        Some(s.to_str().unwrap_or(&s.to_string_lossy()).to_owned())
    });

    let res_1 = target::cpuid(0x8000_0002);
    let res_2 = target::cpuid(0x8000_0003);
    let res_3 = target::cpuid(0x8000_0004);
    // SAFETY:
    // transmute is just changing a [u32; 12] to [u8; 48]. The size remains the
    // same, no invalid types can be created, so it should always be safe.
    let mut name_buf = unsafe {
        transmute::<_, [u8; 48]>([
            res_1.eax, res_1.ebx, res_1.ecx, res_1.edx, res_2.eax, res_2.ebx,
            res_2.ecx, res_2.edx, res_3.eax, res_3.ebx, res_3.ecx, res_3.edx,
        ])
    }
    .to_vec();
    name_buf.push(b'\0');
    let name = CStr::from_bytes_until_nul(&name_buf).map_or(None, |s| {
        Some(s.to_str().unwrap_or(&s.to_string_lossy()).to_owned())
    });

    (vendor, name)
}
