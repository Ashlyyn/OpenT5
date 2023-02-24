use std::{ffi::CStr, mem::transmute};

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
    let res = target::cpuid(0x0);
    let mut vendor_buf =
        unsafe { transmute::<_, [u8; 12]>([res.ebx, res.ecx, res.edx]) }
            .to_vec();
    vendor_buf.push(b'\0');
    let vendor = match CStr::from_bytes_until_nul(&vendor_buf) {
        Ok(s) => Some(s.to_str().unwrap_or(&s.to_string_lossy()).to_string()),
        Err(_) => None,
    };

    let res_1 = target::cpuid(0x80000002);
    let res_2 = target::cpuid(0x80000003);
    let res_3 = target::cpuid(0x80000004);
    let mut name_buf = unsafe {
        transmute::<_, [u8; 48]>([
            res_1.eax, res_1.ebx, res_1.ecx, res_1.edx, res_2.eax, res_2.ebx,
            res_2.ecx, res_2.edx, res_3.eax, res_3.ebx, res_3.ecx, res_3.edx,
        ])
    }
    .to_vec();
    name_buf.push(b'\0');
    let name = match CStr::from_bytes_until_nul(&name_buf) {
        Ok(s) => Some(s.to_str().unwrap_or(&s.to_string_lossy()).to_string()),
        Err(_) => None,
    };

    (vendor, name)
}
