#![allow(unused)]

use std::io::{Cursor, Read, Write};
use std::{io, marker::PhantomData};

use bitbit::{BitReader, BitWriter, MSB};

use crate::bwt_coder::BWTCoder;
use crate::huffman::TreeNode;

pub struct BwtMtfRleHuffmanCoder {
    p: PhantomData<()>,
}

impl BwtMtfRleHuffmanCoder {
    pub fn new() -> Self {
        BwtMtfRleHuffmanCoder { p: PhantomData }
    }

    pub fn encode(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        let bwt_coder = BWTCoder::new();
        let bwt = bwt_coder.encode(bytes)?;

        let mut length_length_frequencies = vec![[0u64; 256]; 256];
        let mut byte_byte_frequencies = vec![[0u64; 256]; 256];

        for window in bwt.windows(4).step_by(2) {
            let &[l1, b1, l2, b2] = window else {
                unreachable!();
            };

            length_length_frequencies[l1 as usize][l2 as usize] += 1;
            byte_byte_frequencies[b1 as usize][b2 as usize] += 1;
        }

        let length_length_trees = length_length_frequencies
            .into_iter()
            .map(|frequencies| TreeNode::build(&frequencies).unwrap())
            .collect::<Vec<_>>();

        let byte_byte_trees = byte_byte_frequencies
            .into_iter()
            .map(|frequencies| TreeNode::build(&frequencies).unwrap())
            .collect::<Vec<_>>();

        let mut output = Vec::new();
        let mut output_cursor = Cursor::new(&mut output);

        output_cursor.write(&bwt.len().to_be_bytes())?;

        let mut writer = BitWriter::new(output_cursor);

        for tree in &length_length_trees {
            tree.encode(&mut writer)?;
        }

        for tree in &byte_byte_trees {
            tree.encode(&mut writer)?;
        }

        let length_length_codes = length_length_trees
            .into_iter()
            .map(|tree| tree.codes())
            .collect::<Vec<_>>();

        let byte_byte_codes = byte_byte_trees
            .into_iter()
            .map(|tree| tree.codes())
            .collect::<Vec<_>>();

        let mut previous_length = 0u8;
        let mut previous_byte = 0u8;

        for window in bwt.chunks(2) {
            let &[length, byte] = window else {
                unreachable!()
            };

            let length_code = length_length_codes[previous_length as usize][length as usize];
            let byte_code = byte_byte_codes[previous_byte as usize][byte as usize];

            length_code.encode(&mut writer)?;
            byte_code.encode(&mut writer)?;

            previous_length = length;
            previous_byte = byte;
        }

        writer.pad_to_byte()?;

        Ok(output)
    }

    pub fn decode(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        let mut input_cursor = Cursor::new(bytes);

        let mut length_bytes = [0u8; 8];
        input_cursor.read(&mut length_bytes)?;
        let length = usize::from_be_bytes(length_bytes);

        let mut reader = BitReader::<_, MSB>::new(input_cursor);

        let mut length_length_trees = Vec::new();

        for _ in 0..256 {
            length_length_trees.push(TreeNode::decode(&mut reader)?);
        }

        let mut byte_byte_trees = Vec::new();

        for _ in 0..256 {
            byte_byte_trees.push(TreeNode::decode(&mut reader)?);
        }

        let mut output = Vec::new();

        let mut previous_length = 0u8;
        let mut previous_byte = 0u8;

        for _ in 0..length / 2 {
            let length =
                length_length_trees[previous_length as usize].decode_symbol(&mut reader)?;

            let byte = byte_byte_trees[previous_byte as usize].decode_symbol(&mut reader)?;

            output.push(length);
            output.push(byte);

            previous_length = length;
            previous_byte = byte;
        }

        let bwt_coder = BWTCoder::new();
        let output = bwt_coder.decode(&output)?;

        Ok(output)
    }
}
