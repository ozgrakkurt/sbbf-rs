#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use x86_64::{contains, insert};

#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(target_arch = "aarch64")]
pub use aarch64::{contains, insert};

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
compile_error!("target is not supported.");
