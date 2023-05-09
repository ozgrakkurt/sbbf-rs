use crate::FilterImpl;

pub struct FallbackFilter;

impl FilterImpl for FallbackFilter {
    unsafe fn contains_unchecked(&self, _buf: *const u8, _len: usize, _hash: u64) -> bool {
        todo!()
    }
    unsafe fn insert_unchecked(&self, _buf: *mut u8, _len: usize, _hash: u64) -> bool {
        todo!()
    }
    fn which(&self) -> &'static str {
        "FallbackFilter"
    }
}
