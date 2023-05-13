use rand::Rng;
use sbbf_rs::{FilterFn, ALIGNMENT, BUCKET_SIZE};
use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    collections::HashSet,
};
use xxhash_rust::xxh3::xxh3_64;

fn run_test(bits_per_key: usize, max_fp: f64) {
    let num_keys = 1_000_000;
    let mut filter = Filter::new(bits_per_key, num_keys);
    let ref_filter = Filter::new(bits_per_key, num_keys);
    let mut rng = rand::thread_rng();

    let mut hashes = HashSet::with_capacity(num_keys);

    // insert hashes
    for i in 0..num_keys {
        let i = rng.gen_range(0..i + 1);
        let hash = xxh3_64(i.to_be_bytes().as_ref());
        filter.insert(hash);
        hashes.insert(hash);
        parquet2::bloom_filter::insert(
            unsafe {
                std::slice::from_raw_parts_mut(ref_filter.buf.ptr, ref_filter.buf.layout.size())
            },
            hash,
        );
        assert!(filter.contains(hash));
    }

    let num_fp_tests = 1_000_000usize;

    let mut fp_count = 0;
    let mut p_count = 0;

    // count false positives
    for i in 0..num_fp_tests {
        let i = rng.gen_range(0..i + 1);
        let hash = xxh3_64(i.to_be_bytes().as_ref());

        let ref_slice =
            unsafe { std::slice::from_raw_parts(ref_filter.buf.ptr, ref_filter.buf.layout.size()) };

        assert_eq!(
            filter.contains(hash),
            parquet2::bloom_filter::is_in_set(ref_slice, hash)
        );

        if filter.contains(hash) {
            p_count += 1;
            if !hashes.contains(&hash) {
                fp_count += 1;
            }
        }
    }

    for h in hashes {
        assert!(filter.contains(h));
    }

    dbg!(fp_count);
    dbg!(p_count);

    let fp_rate = fp_count as f64 / num_fp_tests as f64;

    dbg!(fp_rate);
    assert!(fp_rate < max_fp);

    let ref_slice =
        unsafe { std::slice::from_raw_parts(ref_filter.buf.ptr, ref_filter.buf.layout.size()) };

    let slice = unsafe { std::slice::from_raw_parts(filter.buf.ptr, filter.buf.layout.size()) };

    if ref_slice != slice {
        panic!("bytes don't match parquet2 filter");
    }
}

#[test]
fn test_filter() {
    run_test(24, 0.0002);
    run_test(16, 0.002);
    run_test(8, 0.02);
}

struct Filter {
    filter_fn: FilterFn,
    buf: Buf,
    num_buckets: usize,
}

impl Filter {
    fn new(bits_per_key: usize, num_keys: usize) -> Self {
        let len = (bits_per_key / 8) * num_keys;
        let len = ((len + BUCKET_SIZE / 2) / BUCKET_SIZE) * BUCKET_SIZE;
        Self {
            filter_fn: FilterFn::new(),
            buf: Buf::new(len),
            num_buckets: len / BUCKET_SIZE,
        }
    }

    fn contains(&self, hash: u64) -> bool {
        unsafe {
            self.filter_fn
                .contains(self.buf.ptr, self.num_buckets, hash)
        }
    }

    fn insert(&mut self, hash: u64) {
        unsafe { self.filter_fn.insert(self.buf.ptr, self.num_buckets, hash) }
    }
}

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
