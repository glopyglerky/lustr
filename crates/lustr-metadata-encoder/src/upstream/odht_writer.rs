// Adapted from odht 0.3.1, MIT OR Apache-2.0. See source-slices.json.
// Required writer operations only: original header, allocation, load factor,
// UnHashFn, group probing, insertion and growth for u64 -> u32 definitions.
// Safe fixed-width byte access replaces generic raw pointer views. Original
// SSE2 group matching and word fallback preserve the applicable fast operations.
use super::odht_groups::{GROUP_SIZE, GroupQuery, REFERENCE_GROUP_SIZE};
use alloc::{vec, vec::Vec};

const HEADER: usize = 32;
const ENTRY: usize = 12;
const GROUP: usize = REFERENCE_GROUP_SIZE;
const FACTOR: u16 = ((87 * u16::MAX as usize) / 100) as u16;

pub(crate) struct HashTableOwned {
    bytes: Vec<u8>,
    slots: usize,
    count: usize,
}

impl Default for HashTableOwned {
    fn default() -> Self {
        Self::with_capacity(12)
    }
}

impl HashTableOwned {
    fn with_capacity(count: usize) -> Self {
        // Original Factor::apply_inverse and slots_needed: round up before
        // choosing a power of two, with one full reference group minimum.
        let needed = ((u16::MAX as u128 * count as u128 + FACTOR as u128 - 1) / FACTOR as u128) as usize;
        let slots = needed.checked_next_power_of_two().unwrap().max(GROUP);
        let mut bytes = vec![0; HEADER + slots * ENTRY + slots + GROUP];
        bytes[..4].copy_from_slice(b"ODHT");
        bytes[4..8].copy_from_slice(&[1, 8, 4, HEADER as u8]);
        bytes[16..24].copy_from_slice(&(slots as u64).to_le_bytes());
        bytes[24..28].copy_from_slice(&[0, 0, 0, 2]);
        bytes[28..30].copy_from_slice(&FACTOR.to_le_bytes());
        bytes[HEADER + slots * ENTRY..].fill(0xff);
        Self { bytes, slots, count: 0 }
    }

    #[inline]
    pub fn insert(&mut self, key: &u64, value: &u32) -> Option<u32> {
        let max_count = ((self.slots as u128 * FACTOR as u128) >> 16) as usize;
        if self.count == max_count {
            self.grow();
        }
        let previous = self.insert_raw(*key, *value);
        if previous.is_none() {
            self.count += 1;
            self.bytes[8..16].copy_from_slice(&(self.count as u64).to_le_bytes());
        }
        previous
    }

    #[inline]
    fn insert_raw(&mut self, key: u64, value: u32) -> Option<u32> {
        let mask = self.slots - 1;
        // Original UnHashFn takes the LAST four bytes of the encoded key.
        let hash = (key >> 32) as u32;
        let h2 = (hash >> 25) as u8;
        let mut probe = ProbeSeq::<GROUP_SIZE>::new(hash as usize, mask);
        let metadata = HEADER + self.slots * ENTRY;
        loop {
            let index = probe.index;
            let group = self.bytes[metadata + index..metadata + index + GROUP_SIZE].try_into().unwrap();
            let mut query = GroupQuery::from(group, h2);
            for m in &mut query {
                let slot = (index + m) & mask;
                let entry = HEADER + slot * ENTRY;
                if self.bytes[entry..entry + 8] == key.to_le_bytes() {
                    let old = u32::from_le_bytes(self.bytes[entry + 8..entry + 12].try_into().unwrap());
                    self.bytes[entry + 8..entry + 12].copy_from_slice(&value.to_le_bytes());
                    return Some(old);
                }
            }
            if let Some(first_empty) = query.first_empty() {
                let slot = (index + first_empty) & mask;
                let entry = HEADER + slot * ENTRY;
                self.bytes[entry..entry + 8].copy_from_slice(&key.to_le_bytes());
                self.bytes[entry + 8..entry + 12].copy_from_slice(&value.to_le_bytes());
                self.bytes[metadata + slot] = h2;
                if slot < GROUP {
                    self.bytes[metadata + self.slots + slot] = h2;
                }
                return None;
            }
            probe.advance(mask);
        }
    }

    #[inline(never)]
    #[cold]
    fn grow(&mut self) {
        let mut grown = Self::with_capacity(self.count * 2);
        let metadata = HEADER + self.slots * ENTRY;
        // Original RawIter order is ascending occupied slot order.
        for slot in 0..self.slots {
            if self.bytes[metadata + slot] & 0x80 == 0 {
                let entry = HEADER + slot * ENTRY;
                let key = u64::from_le_bytes(self.bytes[entry..entry + 8].try_into().unwrap());
                let value = u32::from_le_bytes(self.bytes[entry + 8..entry + 12].try_into().unwrap());
                assert!(grown.insert_raw(key, value).is_none());
            }
        }
        grown.count = self.count;
        grown.bytes[8..16].copy_from_slice(&(self.count as u64).to_le_bytes());
        assert!(grown.slots >= 2 * self.slots);
        *self = grown;
    }

    pub fn raw_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

struct ProbeSeq<const GROUP_SIZE: usize> {
    index: usize,
    base: usize,
    chunk_index: usize,
    stride: usize,
}

impl<const GROUP_SIZE: usize> ProbeSeq<GROUP_SIZE> {
    #[inline]
    fn new(h1: usize, mask: usize) -> Self {
        let initial_index = h1 & mask;

        ProbeSeq { index: initial_index, base: initial_index, chunk_index: 0, stride: 0 }
    }

    #[inline]
    fn advance(&mut self, mask: usize) {
        debug_assert!(GROUP_SIZE <= REFERENCE_GROUP_SIZE);
        debug_assert!(REFERENCE_GROUP_SIZE % GROUP_SIZE == 0);

        // The effect of the code in the two branches is
        // identical if GROUP_SIZE==REFERENCE_GROUP_SIZE
        // but the if statement should make it very easy
        // for the compiler to discard the more costly
        // version and only emit the simplified version.

        if GROUP_SIZE == REFERENCE_GROUP_SIZE {
            self.stride += REFERENCE_GROUP_SIZE;
            self.index += self.stride;
            self.index &= mask;
        } else {
            self.chunk_index += GROUP_SIZE;

            if self.chunk_index == REFERENCE_GROUP_SIZE {
                self.chunk_index = 0;
                self.stride += REFERENCE_GROUP_SIZE;
                self.base += self.stride;
            }

            self.index = (self.base + self.chunk_index) & mask;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct OriginalConfig;
    impl odht::Config for OriginalConfig {
        type Key = u64;
        type Value = u32;
        type EncodedKey = [u8; 8];
        type EncodedValue = [u8; 4];
        type H = odht::UnHashFn;
        fn encode_key(key: &u64) -> [u8; 8] {
            key.to_le_bytes()
        }
        fn encode_value(value: &u32) -> [u8; 4] {
            value.to_le_bytes()
        }
        fn decode_key(key: &[u8; 8]) -> u64 {
            u64::from_le_bytes(*key)
        }
        fn decode_value(value: &[u8; 4]) -> u32 {
            u32::from_le_bytes(*value)
        }
    }

    #[test]
    fn original_odht_bytes_and_reader_agree_through_collisions_and_growth() {
        let sets = [
            (0..512u64).map(|v| v.wrapping_mul(0x9e3779b97f4a7c15)).collect::<Vec<_>>(),
            (0..129u64).map(|v| (0xdeadbeefu64 << 32) | v).collect(),
            vec![0, u64::MAX, 1, 1 << 32, 0, u64::MAX, 1],
        ];
        for keys in sets {
            let mut original = odht::HashTableOwned::<OriginalConfig>::default();
            let mut adapted = HashTableOwned::default();
            assert_eq!(adapted.raw_bytes(), original.raw_bytes());
            for (index, key) in keys.iter().enumerate() {
                let value = index as u32;
                assert_eq!(adapted.insert(key, &value), original.insert(key, &value));
                assert_eq!(adapted.raw_bytes(), original.raw_bytes(), "after member {index}");
            }
            let decoded = odht::HashTable::<OriginalConfig, &[u8]>::from_raw_bytes(adapted.raw_bytes()).unwrap();
            for key in keys {
                assert_eq!(decoded.get(&key), original.get(&key));
            }
        }
    }
}
