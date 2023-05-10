use crate::FilterImpl;

pub struct NeonFilter;

impl FilterImpl for NeonFilter {
    #[target_feature(enable = "neon")]
    unsafe fn contains_unchecked(&self, buf: *const u8, len: usize, hash: u64) -> bool {
        todo!()
    }
    #[target_feature(enable = "neon")]
    unsafe fn insert_unchecked(&self, buf: *mut u8, len: usize, hash: u64) {
        todo!()
    }
    fn which(&self) -> &'static str {
        "NeonFilter"
    }
}
