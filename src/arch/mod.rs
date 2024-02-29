#[cfg(all(target_arch = "aarch64", target_endian = "little"))]
mod aarch64;
mod fallback;
#[cfg(all(target_family = "wasm", target_feature = "simd128", target_endian = "little"))]
mod wasm;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod x86;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) fn load() -> &'static dyn crate::FilterImpl {
    cpufeatures::new!(cpuid_avx2, "avx2");
    cpufeatures::new!(cpuid_sse, "sse4.1");

    if cpuid_avx2::get() {
        &x86::Avx2Filter
    } else if cpuid_sse::get() {
        &x86::SseFilter
    } else {
        &fallback::FallbackFilter
    }
}

#[cfg(all(target_arch = "aarch64", target_endian = "little"))]
pub(crate) fn load() -> &'static dyn crate::FilterImpl {
    &aarch64::NeonFilter
}

#[cfg(all(target_family = "wasm", target_feature = "simd128", target_endian = "little"))]
pub(crate) fn load() -> &'static dyn crate::FilterImpl {
    &wasm::WasmFilter
}

#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "x86",
    all(target_arch = "aarch64", target_endian = "little"),
    all(target_family = "wasm", target_feature = "simd128", target_endian = "little")
)))]
pub(crate) fn load() -> &'static dyn crate::FilterImpl {
    &fallback::FallbackFilter
}

#[cfg(any(
    target_arch = "x86_64",
    target_arch = "x86",
    all(target_arch = "aarch64", target_endian = "little"),
    all(target_family = "wasm", target_feature = "simd128", target_endian = "little")
))]
const SALT: [u32; 8] = [
    0x47b6137b, 0x44974d91, 0x8824ad5b, 0xa2b7289d, 0x705495c7, 0x2df1424b, 0x9efc4947, 0x5c6bfb31,
];
