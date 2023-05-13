use rand::distributions::{Distribution, Uniform};
use sbbf_rs::{FilterFn, ALIGNMENT, BUCKET_SIZE};
use std::alloc::{alloc, dealloc, Layout};

const NUM_KEYS: usize = 65536;
const BITS_PER_KEY: usize = 24;

#[test]
fn test_filter() {
    let filter_fn = FilterFn::new();
    dbg!(filter_fn.which());
    let len = (BITS_PER_KEY / 8) * NUM_KEYS;
    let len = ((len + BUCKET_SIZE / 2) / BUCKET_SIZE) * BUCKET_SIZE;
    let buf = Buf::new(len);
    let num_buckets = len / BUCKET_SIZE;
    for i in 0..NUM_KEYS {
        unsafe {
            filter_fn.insert(buf.ptr, num_buckets, i as u64);
        }
    }
    let dist = Uniform::from(0x0..0xffffffffffffffffu64);
    let mut cnt = 0;
    let count = 50000000;
    let mut rng = rand::thread_rng();
    for _ in 0..count {
        let ans = unsafe { filter_fn.contains(buf.ptr, num_buckets, dist.sample(&mut rng)) };
        if ans {
            cnt += 1;
        }
    }
    let rate = cnt as f64 / count as f64;
    dbg!(rate);

    assert!(rate < 0.00019);
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
