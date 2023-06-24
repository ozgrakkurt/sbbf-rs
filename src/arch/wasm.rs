use super::SALT;
use core::arch::wasm::{
    f32x4_convert_u32x4, u32x4, u32x4_add, u32x4_mul, u32x4_shl, u32x4_shr, u32x4_splat,
    u32x4_trunc_sat_f32x4, v128, v128_andnot, v128_any_true, v128_or, v128_store,
};

use crate::FilterImpl;

pub struct WasmFilter;

impl WasmFilter {
    #[inline(always)]
    unsafe fn power_of_two(b: v128) -> v128 {
        let exp = u32x4_add(b, u32x4_splat(127));
        let f = f32x4_convert_u32x4(u32x4_shl(exp, 23));
        u32x4_trunc_sat_f32x4(f)
    }

    #[inline(always)]
    unsafe fn make_mask(hash: u32) -> (v128, v128) {
        let salt = (
            Self::load_v(SALT.as_ptr()),
            Self::load_v(SALT[4..].as_ptr()),
        );
        let hash = u32x4_splat(hash);
        let mut acc = (u32x4_mul(salt.0, hash), u32x4_mul(salt.1, hash));
        acc = (u32x4_shr(acc.0, 27), u32x4_shr(acc.1, 27));
        (Self::power_of_two(acc.0), Self::power_of_two(acc.1))
    }

    #[inline(always)]
    unsafe fn load_v(vals: *const u32) -> v128 {
        u32x4(*vals.add(3), *vals.add(2), *vals.add(1), *vals.add(0))
    }

    #[inline(always)]
    unsafe fn check(mask: v128, bucket: v128) -> bool {
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

        let bucket = bucket as *mut v128;
        v128_store(bucket, c.0);
        v128_store(bucket.add(1), c.1);

        res
    }
    fn which(&self) -> &'static str {
        "WasmFilter"
    }
}

#[cfg(test)]
mod test {
    use super::*;
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
    fn smoke_test_wasm() {
        unsafe {
            let buf = Buf::new(64);

            assert!(!WasmFilter.insert(buf.ptr, 2, 69));
            assert!(WasmFilter.contains(buf.ptr, 2, 69));
            assert!(!WasmFilter.contains(buf.ptr, 2, 12));
            assert!(WasmFilter.insert(buf.ptr, 2, 69));
        }
    }
}
