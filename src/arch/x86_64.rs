use crate::FilterImpl;

pub struct Avx2Filter;

impl FilterImpl for Avx2Filter {
    #[target_feature(enable = "avx2")]
    unsafe fn contains_unchecked(&self, buf: *const u8, len: usize, hash: u32) -> bool {
        todo!()
    }
    #[target_feature(enable = "avx2")]
    unsafe fn insert_unchecked(&self, buf: *mut u8, len: usize, hash: u32) -> bool {
        todo!()
    }
    fn which(&self) -> &'static str {
        "Avx2Filter"
    }
}

pub struct SseFilter;

impl FilterImpl for SseFilter {
    #[target_feature(enable = "sse4.1")]
    unsafe fn contains_unchecked(&self, buf: *const u8, len: usize, hash: u32) -> bool {
        todo!()
    }
    #[target_feature(enable = "sse4.1")]
    unsafe fn insert_unchecked(&self, buf: *mut u8, len: usize, hash: u32) -> bool {
        todo!()
    }

    fn which(&self) -> &'static str {
        "SseFilter"
    }
}
