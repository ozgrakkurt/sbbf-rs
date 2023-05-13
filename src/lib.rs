#![no_std]

mod arch;

const ALIGNMENT: usize = 32;
const BUCKET_SIZE: usize = 32;

/// checks given buffer for alignment and size requirements
/// returns number of buckets in the buffer
/// # Panics
/// panics if buffer doesn't meet requirements
fn check_buf(buf: &[u8]) -> usize {
    assert_eq!(buf.as_ptr().align_offset(ALIGNMENT), 0);
    assert!(!buf.is_empty());
    assert_eq!(buf.len() % BUCKET_SIZE, 0);
    buf.len() / BUCKET_SIZE
}

pub struct FilterRef {}

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

    /// Check if buf contains hash.
    /// # Panics
    /// Panics if the buffer isn't aligned to 32 bytes or
    /// the buffer is empty or the size of the buffer isn't
    /// a multiple of 32
    pub fn contains(&self, buf: &[u8], hash: u64) -> bool {
        let num_buckets = check_buf(buf);
        unsafe {
            self.inner
                .contains_unchecked(buf.as_ptr(), num_buckets, hash)
        }
    }

    /// Insert the hash into the buffer and return true
    /// if it was already in the buffer.
    /// # Panics
    /// Panics if the buffer isn't aligned to 32 bytes or
    /// the buffer is empty or the size of the buffer isn't
    /// a multiple of 32
    pub fn insert(&self, buf: &mut [u8], hash: u64) {
        let num_buckets = check_buf(buf);
        unsafe {
            self.inner
                .insert_unchecked(buf.as_mut_ptr(), num_buckets, hash)
        }
    }

    /// Check if buf contains hash.
    /// num_buckets should be equal to length of the buffer divided by 32.
    /// # Safety
    /// Caller should make sure the buffer is aligned to 32 bytes and
    /// the buffer is non-empty and the size of the buffer is
    /// a multiple of 32
    pub unsafe fn contains_unchecked(&self, buf: *const u8, num_buckets: usize, hash: u64) -> bool {
        self.inner.contains_unchecked(buf, num_buckets, hash)
    }

    /// Insert the hash into the buffer and return true
    /// if it was already in the buffer.
    /// num_buckets should be equal to length of the buffer divided by 32.
    /// # Safety
    /// Caller should make sure the buffer is aligned to 32 bytes and
    /// the buffer is non-empty and the size of the buffer is
    /// a multiple of 32
    pub unsafe fn insert_unchecked(&self, buf: *mut u8, num_buckets: usize, hash: u64) {
        self.inner.insert_unchecked(buf, num_buckets, hash)
    }

    /// Returns a string indicating which internal filter implementation is being used
    pub fn which(&self) -> &'static str {
        self.inner.which()
    }
}

trait FilterImpl {
    unsafe fn contains_unchecked(&self, buf: *const u8, num_buckets: usize, hash: u64) -> bool;
    unsafe fn insert_unchecked(&self, buf: *mut u8, num_buckets: usize, hash: u64);
    fn which(&self) -> &'static str;
}

impl Default for FilterFn {
    fn default() -> Self {
        Self::new()
    }
}
