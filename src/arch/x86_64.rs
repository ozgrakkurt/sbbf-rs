use core::arch::x86_64::{
    __m128i, __m256i, _mm256_mullo_epi32, _mm256_or_si256, _mm256_set1_epi32, _mm256_setr_epi32,
    _mm256_sllv_epi32, _mm256_srli_epi32, _mm256_store_si256, _mm256_testc_si256, _mm_mulhi_epi16,
    _mm_or_si128, _mm_set1_epi32, _mm_set_epi8, _mm_setr_epi16, _mm_shuffle_epi8, _mm_storeu_si128,
    _mm_testc_si128,
};

use crate::{FilterImpl, BUCKET_SIZE};

pub struct Avx2Filter;

impl Avx2Filter {
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn make_mask(&self, hash: u32) -> __m256i {
        let ones = _mm256_set1_epi32(1);
        let rehash = _mm256_setr_epi32(
            0x47b6137bu32 as i32,
            0x44974d91u32 as i32,
            0x8824ad5bu32 as i32,
            0xa2b7289du32 as i32,
            0x705495c7u32 as i32,
            0x2df1424bu32 as i32,
            0x9efc4947u32 as i32,
            0x5c6bfb31u32 as i32,
        );
        let mut hash_data = _mm256_set1_epi32(hash as i32);
        hash_data = _mm256_mullo_epi32(rehash, hash_data);
        hash_data = _mm256_srli_epi32(hash_data, 27u32 as i32);
        _mm256_sllv_epi32(ones, hash_data)
    }
}

impl FilterImpl for Avx2Filter {
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn contains_unchecked(&self, buf: *const u8, len: usize, hash: u64) -> bool {
        let bucket_count = len / BUCKET_SIZE;
        let bucket_idx =
            fastrange_rs::fastrange_32(hash.rotate_left(32) as u32, bucket_count as u32);
        let mask = self.make_mask(hash as u32);
        let bucket = core::mem::transmute::<_, *const __m256i>(buf).add(bucket_idx as usize);
        _mm256_testc_si256(*bucket, mask) != 0
    }
    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn insert_unchecked(&self, buf: *mut u8, len: usize, hash: u64) {
        let bucket_count = len / BUCKET_SIZE;
        let bucket_idx =
            fastrange_rs::fastrange_32(hash.rotate_left(32) as u32, bucket_count as u32);
        let mask = self.make_mask(hash as u32);
        let bucket = core::mem::transmute::<_, *mut __m256i>(buf).add(bucket_idx as usize);
        _mm256_store_si256(bucket, _mm256_or_si256(*bucket, mask));
    }
    fn which(&self) -> &'static str {
        "Avx2Filter"
    }
}

const SSE_BUCKET_SIZE: usize = 16;

pub struct SseFilter;

impl SseFilter {
    #[target_feature(enable = "sse4.1")]
    #[inline]
    unsafe fn make_mask(&self, hash: u32) -> __m128i {
        let rehash1 = _mm_setr_epi16(
            0x47b5u16 as i16,
            0x4497u16 as i16,
            0x8823u16 as i16,
            0xa2b7u16 as i16,
            0x7053u16 as i16,
            0x2df1u16 as i16,
            0x9efcu16 as i16,
            0x5c6bu16 as i16,
        );
        let hash_data = _mm_set1_epi32(hash as i32);
        let h = _mm_mulhi_epi16(rehash1, hash_data);
        _mm_shuffle_epi8(
            _mm_set_epi8(1, 2, 4, 8, 16, 32, 64, -128, 1, 2, 4, 8, 16, 32, 64, -128),
            h,
        )
    }
}

impl FilterImpl for SseFilter {
    #[target_feature(enable = "sse4.1")]
    #[inline]
    unsafe fn contains_unchecked(&self, buf: *const u8, len: usize, hash: u64) -> bool {
        let bucket_count = len / SSE_BUCKET_SIZE;
        let bucket_idx =
            fastrange_rs::fastrange_32(hash.rotate_left(32) as u32, bucket_count as u32);
        let mask = self.make_mask(hash as u32);
        let bucket = core::mem::transmute::<_, *const __m128i>(buf).add(bucket_idx as usize);
        _mm_testc_si128(*bucket, mask) != 0
    }
    #[target_feature(enable = "sse4.1")]
    #[inline]
    unsafe fn insert_unchecked(&self, buf: *mut u8, len: usize, hash: u64) {
        let bucket_count = len / SSE_BUCKET_SIZE;
        let bucket_idx =
            fastrange_rs::fastrange_32(hash.rotate_left(32) as u32, bucket_count as u32);
        let mask = self.make_mask(hash as u32);
        let bucket = core::mem::transmute::<_, *mut __m128i>(buf).add(bucket_idx as usize);
        let bucketvalue = _mm_or_si128(*bucket, mask);
        _mm_storeu_si128(bucket, bucketvalue);
    }
    fn which(&self) -> &'static str {
        "SseFilter"
    }
}
