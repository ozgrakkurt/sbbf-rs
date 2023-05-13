use crate::FilterImpl;

pub struct FallbackFilter;

impl FilterImpl for FallbackFilter {
    unsafe fn contains(&self, _buf: *const u8, _len: usize, _hash: u64) -> bool {
        todo!()
    }
    unsafe fn insert(&self, _buf: *mut u8, _len: usize, _hash: u64) {
        todo!()
    }
    unsafe fn check_and_insert(&self, _buf: *mut u8, _num_buckets: usize, _hash: u64) -> bool {
        todo!()
    }
    fn which(&self) -> &'static str {
        "FallbackFilter"
    }
}
