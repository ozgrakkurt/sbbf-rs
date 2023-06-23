use core::arch::x86_64::{
    __m128i, __m256i, _mm256_load_si256, _mm256_mullo_epi32, _mm256_or_si256, _mm256_set1_epi32,
    _mm256_setr_epi32, _mm256_sllv_epi32, _mm256_srli_epi32, _mm256_store_si256,
    _mm256_testc_si256, _mm_add_epi32, _mm_castsi128_ps, _mm_cvtps_epi32, _mm_mullo_epi32,
    _mm_or_si128, _mm_set1_epi32, _mm_setr_epi32, _mm_slli_epi32, _mm_srli_epi32, _mm_storeu_si128,
    _mm_testc_si128,
};

use super::SALT;
use crate::FilterImpl;

pub struct Avx2Filter;

impl Avx2Filter {
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn make_mask(hash: u32) -> __m256i {
        let salt = _mm256_setr_epi32(
            SALT[0] as i32,
            SALT[1] as i32,
            SALT[2] as i32,
            SALT[3] as i32,
            SALT[4] as i32,
            SALT[5] as i32,
            SALT[6] as i32,
            SALT[7] as i32,
        );
        let mut acc = _mm256_set1_epi32(hash as i32);
        acc = _mm256_mullo_epi32(salt, acc);
        acc = _mm256_srli_epi32(acc, 27);
        _mm256_sllv_epi32(_mm256_set1_epi32(1), acc)
    }
}

impl FilterImpl for Avx2Filter {
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn contains(&self, buf: *const u8, num_buckets: usize, hash: u64) -> bool {
        let bucket_idx =
            fastrange_rs::fastrange_32(hash.rotate_left(32) as u32, num_buckets as u32);
        let mask = Self::make_mask(hash as u32);
        let bucket = (buf as *const __m256i).add(bucket_idx as usize);
        _mm256_testc_si256(_mm256_load_si256(bucket), mask) != 0
    }
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn insert(&self, buf: *mut u8, num_buckets: usize, hash: u64) -> bool {
        let bucket_idx =
            fastrange_rs::fastrange_32(hash.rotate_left(32) as u32, num_buckets as u32);
        let mask = Self::make_mask(hash as u32);
        let bucket = (buf as *mut __m256i).add(bucket_idx as usize);
        let val = _mm256_load_si256(bucket);
        let res = _mm256_testc_si256(val, mask) != 0;
        _mm256_store_si256(bucket, _mm256_or_si256(val, mask));
        res
    }
    fn which(&self) -> &'static str {
        "Avx2Filter"
    }
}

pub struct SseFilter;

impl SseFilter {
    // taken and adapted from https://stackoverflow.com/questions/57454416/sse-integer-2n-powers-of-2-for-32-bit-integers-without-avx2
    #[target_feature(enable = "sse4.1")]
    #[inline]
    unsafe fn power_of_two(b: __m128i) -> __m128i {
        let exp = _mm_add_epi32(b, _mm_set1_epi32(127));
        let f = _mm_castsi128_ps(_mm_slli_epi32(exp, 23));
        _mm_cvtps_epi32(f)
    }

    #[target_feature(enable = "sse4.1")]
    #[inline]
    unsafe fn make_mask(hash: u32) -> (__m128i, __m128i) {
        let salt = (
            _mm_setr_epi32(
                SALT[0] as i32,
                SALT[1] as i32,
                SALT[2] as i32,
                SALT[3] as i32,
            ),
            _mm_setr_epi32(
                SALT[4] as i32,
                SALT[5] as i32,
                SALT[6] as i32,
                SALT[7] as i32,
            ),
        );
        let hash = _mm_set1_epi32(hash as i32);
        let mut acc = (_mm_mullo_epi32(salt.0, hash), _mm_mullo_epi32(salt.1, hash));
        acc = (_mm_srli_epi32(acc.0, 27), _mm_srli_epi32(acc.1, 27));
        (Self::power_of_two(acc.0), Self::power_of_two(acc.1))
    }
}

impl FilterImpl for SseFilter {
    #[target_feature(enable = "sse4.1")]
    #[inline]
    unsafe fn contains(&self, buf: *const u8, num_buckets: usize, hash: u64) -> bool {
        let bucket_idx =
            fastrange_rs::fastrange_32(hash.rotate_left(32) as u32, num_buckets as u32);
        let mask = Self::make_mask(hash as u32);
        let bucket = (buf as *const __m128i).add((bucket_idx * 2) as usize);
        _mm_testc_si128(*bucket, mask.0) != 0 && _mm_testc_si128(*bucket.add(1), mask.1) != 0
    }
    #[target_feature(enable = "sse4.1")]
    #[inline]
    unsafe fn insert(&self, buf: *mut u8, num_buckets: usize, hash: u64) -> bool {
        let bucket_idx =
            fastrange_rs::fastrange_32(hash.rotate_left(32) as u32, num_buckets as u32);
        let mask = Self::make_mask(hash as u32);
        let bucket = (buf as *mut __m128i).add((bucket_idx * 2) as usize);
        _mm_storeu_si128(bucket, _mm_or_si128(*bucket, mask.0));
        let res =
            _mm_testc_si128(*bucket, mask.0) != 0 && _mm_testc_si128(*bucket.add(1), mask.1) != 0;
        _mm_storeu_si128(bucket.add(1), _mm_or_si128(*bucket.add(1), mask.1));
        res
    }
    fn which(&self) -> &'static str {
        "SseFilter"
    }
}

#[cfg(test)]
mod test {
    #[cfg(test)]
    #[macro_use]
    extern crate std;

    use crate::ALIGNMENT;
    use std::alloc::{alloc_zeroed, dealloc, Layout};

    struct Buf {
        ptr: *mut u8,
        layout: Layout,
    }

    impl Buf {
        fn new(len: usize) -> Self {
            let layout = Layout::from_size_align(len, ALIGNMENT).unwrap();
            let ptr = unsafe { alloc_zeroed(layout) };

            Self { layout, ptr }
        }
    }

    impl Drop for Buf {
        fn drop(&mut self) {
            unsafe {
                dealloc(self.ptr, self.layout);
            }
        }
    }

    #[test]
    #[cfg(target_feature = "sse4.1")]
    fn smoke_test_sse() {
        let buf = Buf::new(64);

        assert!(!SseFilter.insert(buf.ptr, 2, 69));
        assert!(SseFilter.contains(buf.ptr, 2, 64));
    }
}
