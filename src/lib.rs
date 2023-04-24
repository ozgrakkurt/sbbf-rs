#![no_std]

mod arch;

use arch::{contains, insert};

const ALIGNMENT: usize = 32;
const BUCKET_SIZE: usize = 32;

pub struct FilterRef<'buf> {
    log_num_buckets: u32,
    directory_mask: u32,
    buf: &'buf [u8],
}

impl<'buf> FilterRef<'buf> {
    pub fn new(buf: &'buf [u8]) -> Self {
        assert_eq!(buf.as_ptr().align_offset(ALIGNMENT), 0);
        assert!(!buf.is_empty());
        assert_eq!(buf.len() % BUCKET_SIZE, 0);

        let num_buckets = buf.len() / BUCKET_SIZE;
        let log_num_buckets = num_buckets.ilog2();

        let num_buckets: u32 = num_buckets.try_into().unwrap();
        let directory_mask = num_buckets - 1;

        Self {
            buf,
            log_num_buckets,
            directory_mask,
        }
    }

    pub fn contains(&self, hash: u32) -> bool {
        unsafe { contains(self.buf, self.log_num_buckets, self.directory_mask, hash) }
    }
}

pub struct FilterMut<'buf> {
    log_num_buckets: u32,
    directory_mask: u32,
    buf: &'buf mut [u8],
}

impl<'buf> FilterMut<'buf> {
    pub fn new(buf: &'buf mut [u8]) -> Self {
        assert_eq!(buf.as_ptr().align_offset(ALIGNMENT), 0);
        assert!(!buf.is_empty());
        assert_eq!(buf.len() % BUCKET_SIZE, 0);

        let num_buckets = buf.len() / BUCKET_SIZE;
        let log_num_buckets = num_buckets.ilog2();

        let num_buckets: u32 = num_buckets.try_into().unwrap();
        let directory_mask = num_buckets - 1;

        Self {
            buf,
            log_num_buckets,
            directory_mask,
        }
    }

    pub fn contains(&self, hash: u32) -> bool {
        unsafe { contains(self.buf, self.log_num_buckets, self.directory_mask, hash) }
    }

    pub fn insert(&mut self, hash: u32) -> bool {
        unsafe { insert(self.buf, self.log_num_buckets, self.directory_mask, hash) }
    }
}
