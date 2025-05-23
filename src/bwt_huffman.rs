use std::{io, marker::PhantomData};

use crate::{bwt_coder::BWTCoder, huffman::HuffmanCoder};

pub struct BWTHuffmanCoder {
    p: PhantomData<()>,
}

impl BWTHuffmanCoder {
    pub fn new() -> Self {
        BWTHuffmanCoder { p: PhantomData }
    }

    pub fn encode(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        let bwt_coder = BWTCoder::new();
        let huffman_coder = HuffmanCoder::new();

        let bwt_encoded = bwt_coder.encode(bytes)?;
        let huffman_encoded = huffman_coder.encode(&bwt_encoded)?;

        Ok(huffman_encoded)
    }

    pub fn decode(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        let bwt_coder = BWTCoder::new();
        let huffman_coder = HuffmanCoder::new();

        let huffman_decoded = huffman_coder.decode(bytes)?;
        let bwt_decoded = bwt_coder.decode(&huffman_decoded)?;

        Ok(bwt_decoded)
    }
}
