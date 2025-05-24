#![allow(unused)]

use std::{
    io::{self, Cursor, Read, Write},
    marker::PhantomData,
    usize,
};

const CHUNK_SIZE: usize = 1024 * 1024 * 8;

pub struct BWTCoder {
    p: PhantomData<()>,
}

impl BWTCoder {
    pub fn new() -> Self {
        BWTCoder { p: PhantomData }
    }

    pub fn encode(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        let mut output = Vec::new();
        let mut writer = Cursor::new(&mut output);

        for chunk in bytes.chunks(CHUNK_SIZE) {
            let (bwt, index) = crate::bwt::bwt(chunk);
            writer.write(&(index as u32).to_be_bytes())?;

            let mtf = crate::mtf::mtf(&bwt);
            let data = mtf;

            let mut curr = data[0];
            let mut len = 1;

            for &byte in &data[1..] {
                if curr == byte && len < 255 {
                    len += 1;
                } else {
                    writer.write(&[len as u8, curr])?;
                    curr = byte;
                    len = 1;
                }
            }

            writer.write(&[len as u8, curr])?;
        }

        Ok(output)
    }

    pub fn decode(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        let mut output = Vec::new();
        let mut reader = Cursor::new(bytes);

        let mut index_bytes = [0u8; 4];

        while reader.read(&mut index_bytes)? == index_bytes.len() {
            let index = u32::from_be_bytes(index_bytes);
            let mut chunk = Vec::new();

            while chunk.len() < CHUNK_SIZE {
                let mut header = [0u8; 2];
                if reader.read(&mut header)? < 2 {
                    break;
                }

                let [len, byte] = header;
                chunk.extend(std::iter::repeat_n(byte, len as usize));
            }

            let data = crate::mtf::imtf(&chunk);
            let data = crate::bwt::ibwt(&data, index as usize);
            output.extend(data);
        }

        Ok(output)
    }
}
