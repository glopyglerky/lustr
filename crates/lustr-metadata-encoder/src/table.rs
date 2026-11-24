use alloc::vec::Vec;
// Copyright the Rust Project Developers. Licensed under MIT OR Apache-2.0.
// Encoder side of rustc_metadata/src/rmeta/table.rs at the declared Rust pin.
// IndexVec/FixedSizeEncoding are supplied by the native fact adapter; the
// original dense/sparse selection and byte layout remain unchanged.
use crate::upstream::mem_encoder::MemEncoder;

const SPARSE_BLOCK_LEN: usize = 64;
const SPARSE_INDEX_BYTES: usize = 12;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Table {
    pub position: usize,
    pub width_and_sparse: usize,
    pub len: usize,
}

#[derive(Default)]
pub(crate) struct TableBuilder {
    width: usize,
    blocks: Vec<[u8; 16]>,
}

impl TableBuilder {
    pub fn set(&mut self, index: usize, value: &[u8]) {
        assert!(value.len() <= 16);
        if value.iter().all(|&b| b == 0) { return; }
        if index >= self.blocks.len() { self.blocks.resize(index + 1, [0; 16]); }
        let block = &mut self.blocks[index];
        assert_eq!(*block, [0; 16], "metadata fact written twice");
        block[..value.len()].copy_from_slice(value);
        let width = 16 - trailing_zeros(block);
        self.width = self.width.max(width);
    }

    pub fn set_position(&mut self, index: usize, position: usize) {
        assert_ne!(position, 0);
        self.set(index, &(position as u64).to_le_bytes());
    }

    pub fn set_array(&mut self, index: usize, position: usize, len: usize) {
        let mut bytes = [0; 16];
        encode_interleaved((position as u64).to_le_bytes(), (len as u64).to_le_bytes(), &mut bytes);
        self.set(index, &bytes);
    }

    pub fn encode(&self, buf: &mut MemEncoder) -> Table {
        let pos = buf.position();
        let width = self.width;
        let len = self.blocks.len();
        let index_bytes = len.div_ceil(SPARSE_BLOCK_LEN) * SPARSE_INDEX_BYTES;
        let non_defaults = if index_bytes < len * width {
            self.blocks.iter().filter(|block| **block != [0; 16]).count()
        } else { len };
        let sparse = len - non_defaults > len / 4
            && u32::try_from(non_defaults).is_ok()
            && index_bytes + non_defaults * width < len * width;
        if sparse {
            let mut base = 0u32;
            for chunk in self.blocks.chunks(SPARSE_BLOCK_LEN) {
                let mut bitmap = 0u64;
                for (i, block) in chunk.iter().enumerate() {
                    if *block != [0; 16] { bitmap |= 1 << i; }
                }
                buf.write_array(bitmap.to_le_bytes());
                buf.write_array(base.to_le_bytes());
                base += bitmap.count_ones();
            }
        }
        for block in &self.blocks {
            if !sparse || *block != [0; 16] {
                buf.write_with(|dest| { *dest = *block; width });
            }
        }
        debug_assert!(width <= usize::MAX >> 1);
        Table { position: pos, width_and_sparse: (width << 1) | usize::from(sparse), len }
    }
}

fn trailing_zeros(x: &[u8]) -> usize { x.iter().rev().take_while(|b| **b == 0).count() }

fn encode_interleaved<const N: usize, const M: usize>(a: [u8; M], b: [u8; M], dest: &mut [u8; N]) {
    assert_eq!(M * 2, N);
    for i in 0..M { dest[2 * i] = a[i]; dest[2 * i + 1] = b[i]; }
}
