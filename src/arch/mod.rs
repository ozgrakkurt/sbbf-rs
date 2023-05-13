use crate::FilterImpl;

#[cfg(target_arch = "aarch64")]
mod aarch64;
mod fallback;
#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
pub(crate) fn load() -> &'static dyn FilterImpl {
    cpufeatures::new!(cpuid_av2, "avx2");
    cpufeatures::new!(cpuid_sse, "sse4.1");

    if cpuid_av2::get() {
        &x86_64::Avx2Filter
    } else if cpuid_sse::get() {
        &x86_64::SseFilter
    } else {
        &fallback::FallbackFilter
    }
}

#[cfg(target_arch = "aarch64")]
pub(crate) fn load() -> &'static dyn FilterImpl {
    &aarch64::NeonFilter
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub(crate) fn load() -> &'static dyn FilterImpl {
    &fallback::FallbackFilter
}

const SALT: [u32; 8] = [
    0x47b6137b, 0x44974d91, 0x8824ad5b, 0xa2b7289d, 0x705495c7, 0x2df1424b, 0x9efc4947, 0x5c6bfb31,
];
