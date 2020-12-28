#[cfg(target_arch = "x86")]
use std::arch::x86::__cpuid;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::__cpuid;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
fn is_pdep_slow() -> bool {
    let leaf0 = unsafe { __cpuid(0) };
    let mut buf = Vec::with_capacity(12);
    buf.extend_from_slice(&leaf0.ebx.to_le_bytes());
    buf.extend_from_slice(&leaf0.edx.to_le_bytes());
    buf.extend_from_slice(&leaf0.ecx.to_le_bytes());

    println!(
        "Detected CPU manufacturer {}",
        String::from_utf8_lossy(&buf)
    );

    if buf.as_slice() == b"AuthenticAMD" || buf.as_slice() == b"HygonGenuine" {
        let leaf1 = unsafe { __cpuid(1) };

        let family = (leaf1.eax >> 8) & 0b1111;
        println!("family {}", family);
        let extended_family = (leaf1.eax >> 20) & 0b11111111;
        println!("extended_family {}", extended_family);

        // Zen, Zen+, and Zen 2 CPUs have very poor PDEP/PEXT performance
        if family == 0xF && (extended_family == 0x8 || extended_family == 0x9) {
            println!("Detected Zen CPU");
            return true;
        }
    }

    false
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
fn is_pdep_slow() -> bool {
    true
}

fn main() {
    if std::env::var(
        "CARGO_FEATURE_DANGEROUSLY_FORCE_ENABLE_PDEP_SINCE_I_REALLY_KNOW_WHAT_IM_DOING",
    )
    .is_ok()
    {
        println!("cargo:rustc-cfg=fast_pdep");
    } else if std::env::var("CARGO_FEATURE_NATIVE_OPTIMIZATIONS").is_ok() {
        println!("Compiling with native optimizations");
        if !is_pdep_slow() {
            println!("cargo:rustc-cfg=fast_pdep");
        }
    }
}
