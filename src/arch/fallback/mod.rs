use crate::FilterImpl;

mod parquet_impl;

pub struct FallbackFilter;

impl FilterImpl for FallbackFilter {
    unsafe fn contains(&self, buf: *const u8, num_buckets: usize, hash: u64) -> bool {
        parquet_impl::check_hash(buf, num_buckets, hash)
    }
    unsafe fn insert(&self, buf: *mut u8, num_buckets: usize, hash: u64) -> bool {
        parquet_impl::insert_hash(buf, num_buckets, hash)
    }
    fn which(&self) -> &'static str {
        "FallbackFilter"
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
    fn smoke_test_fallback() {
        unsafe {
            let buf = Buf::new(64);

            assert!(!FallbackFilter.insert(buf.ptr, 2, 69));
            assert!(FallbackFilter.contains(buf.ptr, 2, 69));
            assert!(!FallbackFilter.contains(buf.ptr, 2, 12));
            assert!(FallbackFilter.insert(buf.ptr, 2, 69));
        }
    }
}
