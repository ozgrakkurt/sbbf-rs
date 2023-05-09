use crate::FilterImpl;

pub struct FallbackFilter;

impl FilterImpl for FallbackFilter {
    unsafe fn contains_unchecked(&self, buf: *const u8, len: usize, hash: u32) -> bool {
        todo!()
    }
    unsafe fn insert_unchecked(&self, buf: *mut u8, len: usize, hash: u32) -> bool {
        todo!()
    }

    fn which(&self) -> &'static str {
        "FallbackFilter"
    }
}
