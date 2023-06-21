use std::alloc::{alloc_zeroed, dealloc, Layout};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::RngCore;
use sbbf_rs::{FilterFn, ALIGNMENT, BUCKET_SIZE};

mod parquet_impl;

const NUM_KEYS: usize = 10_000_000;
const BITS_PER_KEY: usize = 16;

const KEY_RANGE: u64 = 1_000;

fn benchmark_insert(c: &mut Criterion) {
    c.bench_function("parquet2 insert", |b| {
        let mut rng = rand::thread_rng();

        let mut filter = Filter::new(BITS_PER_KEY, NUM_KEYS);
        for _ in 0..NUM_KEYS {
            filter.insert(rng.next_u64() % KEY_RANGE);
        }

        let num = rng.next_u64();
        b.iter(|| {
            black_box({
                parquet2::bloom_filter::insert(black_box(filter.as_mut()), black_box(num));
                parquet2::bloom_filter::is_in_set(black_box(filter.as_bytes()), black_box(num))
            })
        })
    });

    c.bench_function("parquet insert", |b| {
        let mut rng = rand::thread_rng();

        let mut filter = Filter::new(BITS_PER_KEY, NUM_KEYS);
        for _ in 0..NUM_KEYS {
            filter.insert(rng.next_u64() % KEY_RANGE);
        }
        let mut filter = parquet_impl::Sbbf::new(filter.as_mut());

        let num = rng.next_u64();
        b.iter(|| {
            black_box({
                black_box(black_box(&mut filter).insert_hash(black_box(num)));
                black_box(black_box(&filter).check_hash(black_box(num)));
            })
        })
    });

    c.bench_function("sbbf-rs insert", |b| {
        let mut rng = rand::thread_rng();

        let mut filter = Filter::new(BITS_PER_KEY, NUM_KEYS);
        for _ in 0..NUM_KEYS {
            filter.insert(rng.next_u64() % KEY_RANGE);
        }

        let num = rng.next_u64();
        b.iter(|| black_box(black_box(black_box(&mut filter).insert(black_box(num)))))
    });
}

fn benchmark_contains(c: &mut Criterion) {
    c.bench_function("parquet2 contains", |b| {
        let mut rng = rand::thread_rng();

        let mut filter = Filter::new(8, NUM_KEYS);
        for _ in 0..NUM_KEYS {
            filter.insert(rng.next_u64() % KEY_RANGE);
        }

        let num = rng.next_u64();

        b.iter(|| parquet2::bloom_filter::is_in_set(black_box(filter.as_bytes()), black_box(num)))
    });

    c.bench_function("parquet contains", |b| {
        let mut rng = rand::thread_rng();

        let mut filter = Filter::new(BITS_PER_KEY, NUM_KEYS);
        for _ in 0..NUM_KEYS {
            filter.insert(rng.next_u64() % KEY_RANGE);
        }

        let filter = parquet_impl::Sbbf::new(filter.as_mut());

        let num = rng.next_u64();

        b.iter(|| {
            black_box(black_box(&filter).check_hash(black_box(num)));
        })
    });

    c.bench_function("sbbf-rs contains", |b| {
        let mut rng = rand::thread_rng();

        let mut filter = Filter::new(8, NUM_KEYS);
        for _ in 0..NUM_KEYS {
            filter.insert(rng.next_u64() % KEY_RANGE);
        }

        let num = rng.next_u64();

        b.iter(|| {
            black_box(black_box(&filter).contains(black_box(num)));
        })
    });
}

criterion_group!(benches, benchmark_insert, benchmark_contains,);
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

    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.buf.ptr, self.buf.layout.size()) }
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
