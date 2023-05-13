use rand::distributions::{Distribution, Uniform};
use sbbf_rs::{FilterFn, ALIGNMENT, BUCKET_SIZE};
use std::{alloc::{alloc, dealloc, Layout}, collections::HashSet};

#[test]
fn test_filter() {
    let num_hashes = u16::MAX as usize;
    let mut filter = Filter::new(24, num_hashes);
    let dist = Uniform::from(0..u32::MAX/2);
    let mut rng = rand::thread_rng();

    let mut hashes = HashSet::with_capacity(num_hashes);

    // insert hashes
    for _ in 0..num_hashes {
        let hash = dist.sample(&mut rng) as u64;
        filter.insert(hash);
        hashes.insert(hash);
        assert!(filter.contains(hash));
    }

    let num_fp_tests = u16::MAX as usize;

    let mut fp_count = 0;
    let mut p_count = 0;

    let dist = Uniform::from(u32::MAX/2..u32::MAX);
    let mut rng = rand::thread_rng();

    // count false positives
    for _ in 0..num_fp_tests {
        let hash = dist.sample(&mut rng) as u64;
        if filter.contains(hash) {
            p_count += 1;
            if !hashes.contains(&hash) {
                fp_count += 1;
            }
        }
    }

    dbg!(fp_count);
    dbg!(p_count);

    let fp_rate = fp_count as f64 / num_fp_tests as f64;

    dbg!(fp_rate);
}

struct Filter {
    filter_fn: FilterFn,
    buf: Buf,
    num_buckets: usize,
}

impl Filter {
    fn new(bits_per_hash: usize, num_hashes: usize) -> Self {
        let len = (bits_per_hash / 8) * num_hashes;
        let len = ((len + BUCKET_SIZE / 2) / BUCKET_SIZE) * BUCKET_SIZE;
        Self {
            filter_fn: FilterFn::new(),
            buf: Buf::new(len),
            num_buckets: len / BUCKET_SIZE,
        }
    }

    fn contains(&self, hash: u64) -> bool {
        unsafe { self.filter_fn.contains(self.buf.ptr, self.num_buckets, hash) }
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
        let ptr = unsafe { alloc(layout) };

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
