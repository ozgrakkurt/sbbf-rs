#[cfg(not(target_feature = "avx2"))]
compile_error!("target is not supported.");

use core::arch::x86_64::{
    __m256i, _mm256_mullo_epi32, _mm256_or_si256, _mm256_set1_epi32, _mm256_setr_epi32,
    _mm256_sllv_epi32, _mm256_srli_epi32, _mm256_store_si256, _mm256_testc_si256,
};

#[inline(always)]
fn make_mask(hash: u32) -> __m256i {
    unsafe {
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
        hash_data = _mm256_srli_epi32(hash_data, 27);
        _mm256_sllv_epi32(ones, hash_data)
    }
}

#[inline(always)]
pub unsafe fn insert(buf: &mut [u8], log_num_buckets: u32, directory_mask: u32, hash: u32) -> bool {
    let bucket_idx = hash & directory_mask;
    let mask = make_mask(hash >> log_num_buckets);
    let bucket = core::mem::transmute::<_, *mut __m256i>(buf.as_mut_ptr()).add(bucket_idx as usize);
    let res = _mm256_testc_si256(*bucket, mask) != 0;
    _mm256_store_si256(bucket, _mm256_or_si256(*bucket, mask));
    res
}

#[inline(always)]
pub unsafe fn contains(buf: &[u8], log_num_buckets: u32, directory_mask: u32, hash: u32) -> bool {
    let bucket_idx = hash & directory_mask;
    let mask = make_mask(hash >> log_num_buckets);
    let bucket = core::mem::transmute::<_, *const __m256i>(buf.as_ptr()).add(bucket_idx as usize);
    _mm256_testc_si256(*bucket, mask) != 0
}
