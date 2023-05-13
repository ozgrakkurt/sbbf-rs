#![no_std]

mod arch;

pub const ALIGNMENT: usize = 64;
pub const BUCKET_SIZE: usize = 32;

/// This struct gives an interface to filter methods
pub struct FilterFn {
    inner: &'static dyn FilterImpl,
}

impl FilterFn {
    /// Loads a cpu specific optimized implementation of a filter.
    /// Doesn't allocate any memory as filter memory is supposed
    /// to be provided by user in each function call
    pub fn new() -> Self {
        Self {
            inner: arch::load(),
        }
    }

    /// num_buckets should be equal to length of the buffer divided by 32.
    /// # Safety
    /// Caller should make sure the buffer is aligned to [ALIGNMENT] bytes and
    /// the buffer is non-empty. The buffer should have a size of at least
    /// `num_buckets` * [BUCKET_SIZE].
    pub unsafe fn contains(&self, buf: *const u8, num_buckets: usize, hash: u64) -> bool {
        self.inner.contains(buf, num_buckets, hash)
    }

    /// Insert the hash into the buffer
    /// # Safety
    /// Caller should make sure the buffer is aligned to [ALIGNMENT] bytes and
    /// the buffer is non-empty. The buffer should have a size of at least
    /// `num_buckets` * [BUCKET_SIZE].
    pub unsafe fn insert(&self, buf: *mut u8, num_buckets: usize, hash: u64) {
        self.inner.insert(buf, num_buckets, hash)
    }

    /// Returns a string indicating which internal filter implementation is being used
    pub fn which(&self) -> &'static str {
        self.inner.which()
    }
}

trait FilterImpl {
    unsafe fn contains(&self, buf: *const u8, num_buckets: usize, hash: u64) -> bool;
    unsafe fn insert(&self, buf: *mut u8, num_buckets: usize, hash: u64);
    fn which(&self) -> &'static str;
}

impl Default for FilterFn {
    fn default() -> Self {
        Self::new()
    }
}
