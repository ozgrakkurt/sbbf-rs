use crate::FilterImpl;

#[cfg(target_arch = "aarch64")]
mod aarch64;
mod fallback;
#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
pub(crate) fn load() -> Box<dyn FilterImpl> {
    if is_x86_feature_detected!("avx2") {
        Box::new(x86_64::Avx2Filter)
    } else if is_x86_feature_detected!("sse4.1") {
        Box::new(x86_64::SseFilter)
    } else {
        Box::new(fallback::FallbackFilter)
    }
}

#[cfg(target_arch = "aarch64")]
pub(crate) fn load() -> Box<dyn FilterImpl> {
    use core::arch::is_arm_feature_detected;

    if is_arm_feature_detected!("neon") {
        Box::new(aarch64::NeonFilter)
    } else {
        Box::new(fallback::FallbackFilter)
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub(crate) fn load() -> Box<dyn FilterImpl> {
    Box::new(fallback::FallbackFilter)
}
