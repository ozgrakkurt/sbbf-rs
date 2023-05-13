use crate::FilterImpl;

pub struct NeonFilter;

impl NeonFilter {
    #[target_feature(enable = "neon")]
    #[inline]
    unsafe fn make_mask(&self, hash: u32) -> __m256i {
        let salt = uint16x8_t(
            SALT[0],
            SALT[1],
            SALT[2],
            SALT[3],
            SALT[4],
            SALT[5],
            SALT[6],
            SALT[7],
        );
        let mut acc = _mm256_set1_epi32(hash as i32);
        acc = _mm256_mullo_epi32(salt, acc);
        acc = _mm256_srli_epi32(acc, 27);
        _mm256_sllv_epi32(_mm256_set1_epi32(1), acc)
    }
}

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
