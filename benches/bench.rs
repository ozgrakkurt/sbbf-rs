use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    time::Instant,
};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::RngCore;
use sbbf_rs::{FilterFn, ALIGNMENT, BUCKET_SIZE};
use xxhash_rust::xxh3::xxh3_64;

mod parquet_impl;

const NUM_KEYS: usize = 10_000_000;
const BITS_PER_KEY: usize = 8;

const KEY_RANGE: u64 = 500;

fn benchmark_insert(c: &mut Criterion) {
    c.bench_function("parquet2 insert", |b| {
        let mut rng = rand::thread_rng();

        let mut filter = Filter::new(BITS_PER_KEY, NUM_KEYS);
        for _ in 0..NUM_KEYS {
            filter.insert(rng.next_u64() % KEY_RANGE);
        }

        b.iter_custom(|iters| {
            let num = rng.next_u64() % KEY_RANGE;
            let start = Instant::now();
            for _ in 0..iters {
                black_box(parquet2::bloom_filter::insert(filter.as_mut(), num));
            }
            start.elapsed()
        })
    });

    c.bench_function("parquet insert", |b| {
        let mut rng = rand::thread_rng();

        let mut filter = Filter::new(BITS_PER_KEY, NUM_KEYS);
        for _ in 0..NUM_KEYS {
            filter.insert(rng.next_u64() % KEY_RANGE);
        }
        let mut filter = parquet_impl::Sbbf::new(filter.as_mut());

        b.iter_custom(|iters| {
            let num = rng.next_u64() % KEY_RANGE;
            let start = Instant::now();
            for _ in 0..iters {
                black_box(filter.insert_hash(num));
            }
            start.elapsed()
        })
    });

    c.bench_function("sbbf-rs insert", |b| {
        let mut rng = rand::thread_rng();

        let mut filter = Filter::new(BITS_PER_KEY, NUM_KEYS);
        for _ in 0..NUM_KEYS {
            filter.insert(rng.next_u64() % KEY_RANGE);
        }

        b.iter_custom(|iters| {
            let num = rng.next_u64() % KEY_RANGE;
            let start = Instant::now();
            for _ in 0..iters {
                black_box(filter.insert(num));
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
            filter.insert(rng.next_u64() % KEY_RANGE);
        }

        b.iter_custom(|iters| {
            let num = rng.next_u64() % KEY_RANGE;
            let start = Instant::now();
            for _ in 0..iters {
                black_box(parquet2::bloom_filter::is_in_set(filter.as_mut(), num));
            }
            start.elapsed()
        })
    });

    c.bench_function("parquet contains", |b| {
        let mut rng = rand::thread_rng();

        let mut filter = Filter::new(BITS_PER_KEY, NUM_KEYS);
        for _ in 0..NUM_KEYS {
            filter.insert(rng.next_u64() % KEY_RANGE);
        }

        let filter = parquet_impl::Sbbf::new(filter.as_mut());

        b.iter_custom(|iters| {
            let num = rng.next_u64() % KEY_RANGE;
            let start = Instant::now();
            for _ in 0..iters {
                black_box(filter.check_hash(num));
            }
            start.elapsed()
        })
    });

    c.bench_function("sbbf-rs contains", |b| {
        let mut rng = rand::thread_rng();

        let mut filter = Filter::new(8, NUM_KEYS);
        for _ in 0..NUM_KEYS {
            filter.insert(rng.next_u64() % KEY_RANGE);
        }

        b.iter_custom(|iters| {
            let num = rng.next_u64() % KEY_RANGE;
            let start = Instant::now();
            for _ in 0..iters {
                black_box(filter.contains(num));
            }
            start.elapsed()
        })
    });
}

fn benchmark_realistic(c: &mut Criterion) {
    let mut rng = rand::thread_rng();

    let setup_data = (0..NUM_KEYS)
        .map(|_| {
            let mut dest = [0; 16];
            rng.fill_bytes(&mut dest);
            dest
        })
        .collect::<Vec<_>>();

    let bench_data = (0..NUM_KEYS)
        .map(|i| {
            let mut dest = [0; 16];
            if rand::random() {
                rng.fill_bytes(&mut dest);
            } else {
                dest.copy_from_slice(&setup_data[i]);
            }
            dest
        })
        .collect::<Vec<_>>();

    let make_filter = || {
        let mut filter = Filter::new(8, NUM_KEYS);
        for key in setup_data.iter() {
            filter.insert(xxh3_64(key));
        }

        filter
    };

    c.bench_function("parquet2 contains realistic", |b| {
        let filter = make_filter();

        let mut index = 0;

        b.iter(|| {
            index = (index + 1) % bench_data.len();
            let key = bench_data[index];
            let hash = xxh3_64(&key);
            black_box(parquet2::bloom_filter::is_in_set(filter.as_bytes(), hash))
        })
    });

    c.bench_function("parquet contains realistic", |b| {
        let filter = make_filter();

        let mut index = 0;

        let filter = parquet_impl::Sbbf::new(filter.as_bytes());

        b.iter(|| {
            index = (index + 1) % bench_data.len();
            let key = bench_data[index];
            let hash = xxh3_64(&key);
            black_box(filter.check_hash(hash))
        })
    });

    c.bench_function("sbbf-rs contains realistic", |b| {
        let filter = make_filter();

        let mut index = 0;

        b.iter(|| {
            index = (index + 1) % bench_data.len();
            let key = bench_data[index];
            let hash = xxh3_64(&key);
            black_box(filter.contains(hash))
        })
    });

    c.bench_function("parquet2 insert realistic", |b| {
        let mut filter = make_filter();

        let mut index = 0;

        b.iter(|| {
            index = (index + 1) % bench_data.len();
            let key = bench_data[index];
            let hash = xxh3_64(&key);
            black_box(parquet2::bloom_filter::insert(filter.as_mut(), hash))
        })
    });

    c.bench_function("parquet insert realistic", |b| {
        let filter = make_filter();

        let mut index = 0;

        let mut filter = parquet_impl::Sbbf::new(filter.as_bytes());

        b.iter(|| {
            index = (index + 1) % bench_data.len();
            let key = bench_data[index];
            let hash = xxh3_64(&key);
            black_box(filter.insert_hash(hash))
        })
    });

    c.bench_function("sbbf-rs insert realistic", |b| {
        let mut filter = make_filter();

        let mut index = 0;

        b.iter(|| {
            index = (index + 1) % bench_data.len();
            let key = bench_data[index];
            let hash = xxh3_64(&key);
            black_box(filter.insert(hash))
        })
    });
}

criterion_group!(
    benches,
    benchmark_insert,
    benchmark_contains,
    benchmark_realistic
);
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
