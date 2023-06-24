use super::SALT;
use core::arch::wasm::{
    u32x4, u32x4_mul, u32x4_shl, u32x4_shr, u32x4_splat, v128, v128_andnot, v128_any_true, v128_or,
    v128_store,
};

use crate::FilterImpl;

pub struct WasmFilter;

impl WasmFilter {
    #[inline(always)]
    unsafe fn make_mask(hash: u32) -> (v128, v128) {
        let salt = (
            Self::load_v(SALT.as_ptr()),
            Self::load_v(SALT[4..].as_ptr()),
        );
        let hash = u32x4_splat(hash);
        let mut acc = (u32x4_mul(salt.0, hash), u32x4_mul(salt.1, hash));
        acc = (u32x4_shr(acc.0, 27), u32x4_shr(acc.1, 27));
        let ones = u32x4_splat(1);
        (u32x4_shl(ones, acc.0), u32x4_shl(ones, acc.1))
    }

    #[inline(always)]
    unsafe fn load_v(vals: *const u32) -> v128 {
        u32x4(vals[3], vals[2], vals[1], vals[0])
    }

    #[inline(always)]
    unsafe fn check(mask: u32x4, bucket: u32x4) -> bool {
        !v128_any_true(v128_andnot(mask, bucket))
    }
}

impl FilterImpl for WasmFilter {
    #[inline(always)]
    unsafe fn contains(&self, buf: *const u8, num_buckets: usize, hash: u64) -> bool {
        let bucket_idx =
            fastrange_rs::fastrange_32(hash.rotate_left(32) as u32, num_buckets as u32);
        let mask = Self::make_mask(hash as u32);
        let bucket = (buf as *const u32).add((bucket_idx * 8) as usize);

        let bucket = (Self::load_v(bucket), Self::load_v(bucket.add(4)));

        Self::check(mask.0, bucket.0) && Self::check(mask.1, bucket.1)
    }
    #[inline(always)]
    unsafe fn insert(&self, buf: *mut u8, num_buckets: usize, hash: u64) -> bool {
        let bucket_idx =
            fastrange_rs::fastrange_32(hash.rotate_left(32) as u32, num_buckets as u32);
        let mask = Self::make_mask(hash as u32);
        let bucket = (buf as *mut u32).add((bucket_idx * 8) as usize);
        let val = (Self::load_v(bucket), Self::load_v(bucket.add(4)));
        let res = Self::check(mask.0, val.0) && Self::check(mask.1, val.1);
        let c = (v128_or(val.0, mask.0), v128_or(val.1, mask.1));
        v128_store(bucket, c.0);
        v128_store(bucket.add(4), c.1);

        res
    }
    fn which(&self) -> &'static str {
        "WasmFilter"
    }
}
