// copy-pasted from https://github.com/apache/arrow-rs/blob/master/parquet/src/bloom_filter/mod.rs
// modified slightly

// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! Bloom filter implementation specific to Parquet, as described
//! in the [spec](https://github.com/apache/parquet-format/blob/master/BloomFilter.md).

#![allow(dead_code)]

/// Salt as defined in the [spec](https://github.com/apache/parquet-format/blob/master/BloomFilter.md#technical-approach).
const SALT: [u32; 8] = [
    0x47b6137b_u32,
    0x44974d91_u32,
    0x8824ad5b_u32,
    0xa2b7289d_u32,
    0x705495c7_u32,
    0x2df1424b_u32,
    0x9efc4947_u32,
    0x5c6bfb31_u32,
];

/// Each block is 256 bits, broken up into eight contiguous "words", each consisting of 32 bits.
/// Each word is thought of as an array of bits; each bit is either "set" or "not set".
#[derive(Debug, Copy, Clone)]
struct Block([u32; 8]);
impl Block {
    const ZERO: Block = Block([0; 8]);

    /// takes as its argument a single unsigned 32-bit integer and returns a block in which each
    /// word has exactly one bit set.
    #[inline(always)]
    fn mask(x: u32) -> Self {
        let mut result = [0_u32; 8];
        for i in 0..8 {
            // wrapping instead of checking for overflow
            let y = x.wrapping_mul(SALT[i]);
            let y = y >> 27;
            result[i] = 1 << y;
        }
        Self(result)
    }

    #[inline(always)]
    #[cfg(target_endian = "little")]
    fn to_le_bytes(self) -> [u8; 32] {
        self.to_ne_bytes()
    }

    #[inline(always)]
    #[cfg(not(target_endian = "little"))]
    fn to_le_bytes(self) -> [u8; 32] {
        self.swap_bytes().to_ne_bytes()
    }

    #[inline(always)]
    fn to_ne_bytes(self) -> [u8; 32] {
        unsafe { core::mem::transmute(self) }
    }

    #[inline(always)]
    #[cfg(not(target_endian = "little"))]
    fn swap_bytes(mut self) -> Self {
        self.0.iter_mut().for_each(|x| *x = x.swap_bytes());
        self
    }

    /// setting every bit in the block that was also set in the result from mask
    #[inline(always)]
    fn insert(&mut self, hash: u32) {
        let mask = Self::mask(hash);
        for i in 0..8 {
            self.0[i] |= mask.0[i];
        }
    }

    /// returns true when every bit that is set in the result of mask is also set in the block.
    #[inline(always)]
    fn check(&self, hash: u32) -> bool {
        let mask = Self::mask(hash);
        for i in 0..8 {
            if self.0[i] & mask.0[i] == 0 {
                return false;
            }
        }
        true
    }

    #[inline(always)]
    fn load(buf: &[u8]) -> Self {
        let mut block = Block::ZERO;

        for (i, word) in buf.chunks_exact(4).enumerate() {
            block.0[i] = u32::from_le_bytes(word.try_into().unwrap());
        }

        block
    }

    #[inline(always)]
    fn store(&self, buf: &mut [u8]) {
        buf.copy_from_slice(self.to_le_bytes().as_slice())
    }
}

#[inline(always)]
pub unsafe fn insert_hash(buf: *mut u8, num_buckets: usize, hash: u64) -> bool {
    let block_idx = hash_to_block_index(num_buckets, hash);

    let buf = core::slice::from_raw_parts_mut(buf.add(block_idx * 32), 32);

    let mut block = Block::load(buf);

    let res = block.check(hash as u32);
    block.insert(hash as u32);

    block.store(buf);

    res
}

#[inline(always)]
pub unsafe fn check_hash(buf: *const u8, num_buckets: usize, hash: u64) -> bool {
    let block_idx = hash_to_block_index(num_buckets, hash);

    let buf = core::slice::from_raw_parts(buf.add(block_idx * 32), 32);

    let block = Block::load(buf);

    block.check(hash as u32)
}

#[inline(always)]
fn hash_to_block_index(num_buckets: usize, hash: u64) -> usize {
    // unchecked_mul is unstable, but in reality this is safe, we'd just use saturating mul
    // but it will not saturate
    (((hash >> 32).saturating_mul(num_buckets as u64)) >> 32) as usize
}
