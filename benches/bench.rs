use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    time::Instant,
};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use probabilistic_collections::bloom::BloomFilter;
use rand::RngCore;
use sbbf_rs::{FilterFn, ALIGNMENT, BUCKET_SIZE};
use xxhash_rust::xxh3::xxh3_64;

const NUM_KEYS: usize = 1_000_000;
const BITS_PER_KEY: usize = 8;
const FPP: f64 = 0.01;

fn benchmark_insert(c: &mut Criterion) {
    c.bench_function("parquet2 insert", |b| {
        let mut rng = rand::thread_rng();

        let mut filter = Filter::new(BITS_PER_KEY, NUM_KEYS);
        for _ in 0..NUM_KEYS {
            parquet2::bloom_filter::insert(filter.as_mut(), rng.next_u64());
        }

        b.iter_custom(|iters| {
            let num = rng.next_u64();
            let start = Instant::now();
            for _ in 0..iters {
                black_box(parquet2::bloom_filter::insert(
                    filter.as_mut(),
                    xxh3_64(&num.to_be_bytes()),
                ));
            }
            start.elapsed()
        })
    });

    c.bench_function("sbbf-rs insert", |b| {
        let mut rng = rand::thread_rng();

        let mut filter = Filter::new(BITS_PER_KEY, NUM_KEYS);
        for _ in 0..NUM_KEYS {
            filter.insert(rng.next_u64());
        }

        b.iter_custom(|iters| {
            let num = rng.next_u64();
            let start = Instant::now();
            for _ in 0..iters {
                black_box(filter.insert(xxh3_64(&num.to_be_bytes())));
            }
            start.elapsed()
        })
    });

    c.bench_function("prob-col insert", |b| {
        let mut rng = rand::thread_rng();

        let mut filter = BloomFilter::<u64>::new(NUM_KEYS, FPP);
        for _ in 0..NUM_KEYS {
            filter.insert(&rng.next_u64());
        }

        b.iter_custom(|iters| {
            let num = rng.next_u64();
            let start = Instant::now();
            for _ in 0..iters {
                black_box(filter.insert(&num));
            }
            start.elapsed()
        })
    });
}

fn benchmark_contains(c: &mut Criterion) {
    c.bench_function("parquet2 contains", |b| {
        let mut rng = rand::thread_rng();

        let mut filter = Filter::new(8, NUM_KEYS);
        for _ in 0..NUM_KEYS {
            parquet2::bloom_filter::insert(filter.as_mut(), rng.next_u64());
        }

        b.iter_custom(|iters| {
            let num = rng.next_u64();
            let start = Instant::now();
            for _ in 0..iters {
                black_box(parquet2::bloom_filter::is_in_set(
                    filter.as_mut(),
                    xxh3_64(&num.to_be_bytes()),
                ));
            }
            start.elapsed()
        })
    });

    c.bench_function("sbbf-rs contains", |b| {
        let mut rng = rand::thread_rng();

        let mut filter = Filter::new(8, NUM_KEYS);
        for _ in 0..NUM_KEYS {
            filter.insert(rng.next_u64());
        }

        b.iter_custom(|iters| {
            let num = rng.next_u64();
            let start = Instant::now();
            for _ in 0..iters {
                black_box(filter.contains(xxh3_64(&num.to_be_bytes())));
            }
            start.elapsed()
        })
    });

    c.bench_function("prob-col contains", |b| {
        let mut rng = rand::thread_rng();

        let mut filter = BloomFilter::<u64>::new(NUM_KEYS, FPP);
        for _ in 0..NUM_KEYS {
            filter.insert(&rng.next_u64());
        }

        b.iter_custom(|iters| {
            let num = rng.next_u64();
            let start = Instant::now();
            for _ in 0..iters {
                black_box(filter.contains(&num));
            }
            start.elapsed()
        })
    });
}

criterion_group!(benches, benchmark_insert, benchmark_contains);
criterion_main!(benches);

struct Filter {
    filter_fn: FilterFn,
    buf: Buf,
    num_buckets: usize,
}

impl Filter {
    #[inline(always)]
    fn new(bits_per_key: usize, num_keys: usize) -> Self {
        let len = (bits_per_key / 8) * num_keys;
        let len = ((len + BUCKET_SIZE / 2) / BUCKET_SIZE) * BUCKET_SIZE;
        Self {
            filter_fn: FilterFn::new(),
            buf: Buf::new(len),
            num_buckets: len / BUCKET_SIZE,
        }
    }

    #[inline(always)]
    fn contains(&self, hash: u64) -> bool {
        unsafe {
            self.filter_fn
                .contains(self.buf.ptr, self.num_buckets, hash)
        }
    }

    #[inline(always)]
    fn insert(&mut self, hash: u64) {
        unsafe { self.filter_fn.insert(self.buf.ptr, self.num_buckets, hash) };
    }

    #[inline(always)]
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.buf.ptr, self.buf.layout.size()) }
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
