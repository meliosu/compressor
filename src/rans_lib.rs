#![allow(unused)]

use std::io;

use rans::{RansEncSymbol, RansEncoder, byte_encoder::ByteRansEncSymbol};

pub struct AnsLibraryCoder;

impl AnsLibraryCoder {
    pub fn new() -> Self {
        Self
    }

    pub fn encode(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        todo!()
    }

    pub fn decode(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        todo!()
    }
}
