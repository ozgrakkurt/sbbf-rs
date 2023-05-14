use crate::{FilterImpl, BUCKET_SIZE};

mod parquet2_impl;

pub struct FallbackFilter;

impl FilterImpl for FallbackFilter {
    unsafe fn contains(&self, buf: *const u8, num_buckets: usize, hash: u64) -> bool {
        parquet2_impl::is_in_set(
            unsafe { core::slice::from_raw_parts(buf, num_buckets * BUCKET_SIZE) },
            hash,
        )
    }
    unsafe fn insert(&self, buf: *mut u8, num_buckets: usize, hash: u64) -> bool {
        parquet2_impl::insert(
            unsafe { core::slice::from_raw_parts_mut(buf, num_buckets * BUCKET_SIZE) },
            hash,
        )
    }
    fn which(&self) -> &'static str {
        "FallbackFilter"
    }
}
