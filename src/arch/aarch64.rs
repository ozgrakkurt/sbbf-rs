use super::SALT;
use core::arch::aarch64::{
    uint32x4_t, vandq_u32, vceqq_u32, vld1q_u32, vminvq_u32, vmulq_u32, vorrq_u32,
    vreinterpretq_s32_u32, vshlq_u32, vshrq_n_u32,
};

use crate::FilterImpl;

pub struct NeonFilter;

impl NeonFilter {
    #[target_feature(enable = "neon")]
    #[inline]
    unsafe fn make_mask(hash: u32) -> (uint32x4_t, uint32x4_t) {
        let salt = (vld1q_u32(SALT.as_ptr()), vld1q_u32(SALT[4..].as_ptr()));
        let hash = vld1q_u32([hash, hash, hash, hash].as_ptr());
        let mut acc = (vmulq_u32(salt.0, hash), vmulq_u32(salt.1, hash));
        acc = (vshrq_n_u32(acc.0, 27), vshrq_n_u32(acc.1, 27));
        let ones = vld1q_u32([1, 1, 1, 1].as_ptr());
        (
            vshlq_u32(ones, vreinterpretq_s32_u32(acc.0)),
            vshlq_u32(ones, vreinterpretq_s32_u32(acc.1)),
        )
    }

    #[target_feature(enable = "neon")]
    #[inline]
    unsafe fn is_eq(a: uint32x4_t, b: uint32x4_t) -> bool {
        vminvq_u32(vceqq_u32(a, b)) == 0xffffffff
    }

    #[target_feature(enable = "neon")]
    #[inline]
    unsafe fn check(mask: uint32x4_t, bucket: uint32x4_t) -> bool {
        Self::is_eq(mask, vandq_u32(mask, bucket))
    }
}

impl FilterImpl for NeonFilter {
    #[target_feature(enable = "neon")]
    #[inline]
    unsafe fn contains(&self, buf: *const u8, num_buckets: usize, hash: u64) -> bool {
        let bucket_idx =
            fastrange_rs::fastrange_32(hash.rotate_left(32) as u32, num_buckets as u32);
        let mask = Self::make_mask(hash as u32);
        let bucket = (buf as *const uint32x4_t).add((bucket_idx * 2) as usize);

        Self::check(mask.0, *bucket) && Self::check(mask.1, *bucket.add(1))
    }
    #[target_feature(enable = "neon")]
    #[inline]
    unsafe fn insert(&self, buf: *mut u8, num_buckets: usize, hash: u64) -> bool {
        let bucket_idx =
            fastrange_rs::fastrange_32(hash.rotate_left(32) as u32, num_buckets as u32);
        let mask = Self::make_mask(hash as u32);
        let bucket = (buf as *mut uint32x4_t).add((bucket_idx * 2) as usize);
        let res = Self::check(mask.0, *bucket) && Self::check(mask.1, *bucket.add(1));
        *bucket = vorrq_u32(*bucket, mask.0);
        *bucket.add(1) = vorrq_u32(*bucket.add(1), mask.1);
        res
    }
    fn which(&self) -> &'static str {
        "NeonFilter"
    }
}
