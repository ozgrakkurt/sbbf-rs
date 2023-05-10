use crate::FilterImpl;

#[cfg(target_arch = "aarch64")]
mod aarch64;
mod fallback;
#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
pub(crate) fn load() -> &'static dyn FilterImpl {
    if is_x86_feature_detected!("avx2") {
        &x86_64::Avx2Filter
    } else if is_x86_feature_detected!("sse4.1") {
        &x86_64::SseFilter
    } else {
        &fallback::FallbackFilter
    }
}

#[cfg(target_arch = "aarch64")]
pub(crate) fn load() -> &'static dyn FilterImpl {
    use core::arch::is_arm_feature_detected;

    if is_arm_feature_detected!("neon") {
        &aarch64::NeonFilter
    } else {
        &fallback::FallbackFilter
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub(crate) fn load() -> &'static dyn FilterImpl {
    &fallback::FallbackFilter
}
