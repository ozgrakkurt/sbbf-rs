#![no_std]

mod arch;

pub const ALIGNMENT: usize = 64;
pub const BUCKET_SIZE: usize = 32;

/// This struct gives an interface to filter methods
#[cfg(not(target_arch = "aarch64"))]
pub struct FilterFn {
    inner: &'static dyn FilterImpl,
}

#[cfg(not(target_arch = "aarch64"))]
impl FilterFn {
    /// Loads a cpu specific optimized implementation of a split block bloom filter.
    /// Doesn't allocate any memory.
    pub fn new() -> Self {
        Self {
            inner: arch::load(),
        }
    }

    /// Check if filter bits in `buf` contain `hash`.
    /// # Safety
    /// Caller should make sure the buffer is aligned to [ALIGNMENT] bytes.
    /// The buffer should have a size of at least `num_buckets` * [BUCKET_SIZE].
    /// `num_buckets` has to be bigger than zero.
    #[inline(always)]
    pub unsafe fn contains(&self, buf: *const u8, num_buckets: usize, hash: u64) -> bool {
        self.inner.contains(buf, num_buckets, hash)
    }

    /// Insert `hash` into the filter bits inside `buf`.
    /// Returns true if `hash` was already in the filter bits inside `buf`.
    /// # Safety
    /// Caller should make sure the buffer is aligned to [ALIGNMENT] bytes.
    /// The buffer should have a size of at least `num_buckets` * [BUCKET_SIZE].
    /// `num_buckets` has to be bigger than zero.
    #[inline(always)]
    pub unsafe fn insert(&self, buf: *mut u8, num_buckets: usize, hash: u64) -> bool {
        self.inner.insert(buf, num_buckets, hash)
    }

    /// Returns a string indicating which internal filter implementation is being used
    pub fn which(&self) -> &'static str {
        self.inner.which()
    }
}

#[cfg(target_arch = "aarch64")]
use arch::NeonFilter;

/// This struct gives an interface to filter methods
#[cfg(target_arch = "aarch64")]
pub struct FilterFn {}

#[cfg(target_arch = "aarch64")]
impl FilterFn {
    /// Loads a cpu specific optimized implementation of a split block bloom filter.
    /// Doesn't allocate any memory.
    pub fn new() -> Self {
        Self {}
    }

    /// Check if filter bits in `buf` contain `hash`.
    /// # Safety
    /// Caller should make sure the buffer is aligned to [ALIGNMENT] bytes.
    /// The buffer should have a size of at least `num_buckets` * [BUCKET_SIZE].
    /// `num_buckets` has to be bigger than zero.
    #[inline(always)]
    pub unsafe fn contains(&self, buf: *const u8, num_buckets: usize, hash: u64) -> bool {
        NeonFilter.contains(buf, num_buckets, hash)
    }

    /// Insert `hash` into the filter bits inside `buf`.
    /// Returns true if `hash` was already in the filter bits inside `buf`.
    /// # Safety
    /// Caller should make sure the buffer is aligned to [ALIGNMENT] bytes.
    /// The buffer should have a size of at least `num_buckets` * [BUCKET_SIZE].
    /// `num_buckets` has to be bigger than zero.
    #[inline(always)]
    pub unsafe fn insert(&self, buf: *mut u8, num_buckets: usize, hash: u64) -> bool {
        NeonFilter.insert(buf, num_buckets, hash)
    }

    /// Returns a string indicating which internal filter implementation is being used
    pub fn which(&self) -> &'static str {
        NeonFilter.which()
    }
}

trait FilterImpl {
    unsafe fn contains(&self, buf: *const u8, num_buckets: usize, hash: u64) -> bool;
    unsafe fn insert(&self, buf: *mut u8, num_buckets: usize, hash: u64) -> bool;

    fn which(&self) -> &'static str;
}

impl Default for FilterFn {
    fn default() -> Self {
        Self::new()
    }
}
