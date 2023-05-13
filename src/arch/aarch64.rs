use super::SALT;
use core::arch::aarch64::{
    uint32x4_t, vbicq_u32, vld1q_u32, vmulq_u32, vorrq_u32, vreinterpretq_s32_u32, vshlq_u32,
    vshrq_n_u32, vst1q_u32,
};

use crate::FilterImpl;

pub struct NeonFilter;

impl NeonFilter {
    #[target_feature(enable = "neon")]
    #[inline]
    unsafe fn make_mask(&self, hash: u32) -> (uint32x4_t, uint32x4_t) {
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
    unsafe fn check(&self, bucket: uint32x4_t, mask: uint32x4_t) -> bool {
        let an = vbicq_u32(mask, bucket);
        let mut vals = [0, 0];
        vst1q_u32(vals.as_mut_ptr(), an);
        vals[0] != 0 && vals[1] != 0
    }
}

impl FilterImpl for NeonFilter {
    #[target_feature(enable = "neon")]
    unsafe fn contains_unchecked(&self, buf: *const u8, num_buckets: usize, hash: u64) -> bool {
        let bucket_idx =
            fastrange_rs::fastrange_32(hash.rotate_left(32) as u32, num_buckets as u32);
        let mask = self.make_mask(hash as u32);
        let bucket = (buf as *const uint32x4_t).add((bucket_idx * 2) as usize);
        self.check(*bucket, mask.0) && self.check(*bucket.add(1), mask.1)
    }
    #[target_feature(enable = "neon")]
    unsafe fn insert_unchecked(&self, buf: *mut u8, num_buckets: usize, hash: u64) {
        let bucket_idx =
            fastrange_rs::fastrange_32(hash.rotate_left(32) as u32, num_buckets as u32);
        let mask = self.make_mask(hash as u32);
        let bucket = (buf as *mut uint32x4_t).add((bucket_idx * 2) as usize);
        *bucket = vorrq_u32(*bucket, mask.0);
        *bucket.add(1) = vorrq_u32(*bucket.add(1), mask.1);
    }
    fn which(&self) -> &'static str {
        "NeonFilter"
    }
}
