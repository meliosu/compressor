#![allow(unused)]

use std::{
    io::{self, Cursor, Read, Write},
    marker::PhantomData,
};

const CHUNK_SIZE: usize = 32768;

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

        writer.write(&bytes.chunks(CHUNK_SIZE).count().to_be_bytes())?;

        for chunk in bytes.chunks(CHUNK_SIZE) {
            let (bwt, index) = crate::bwt::bwt(chunk);
            writer.write(&index.to_be_bytes())?;

            let mut curr = 0;
            let mut len = 0;

            for &byte in &bwt {
                if len == 0 {
                    curr = byte;
                    len = 1;
                    continue;
                }

                if curr == byte && len < 256 {
                    len += 1;
                    continue;
                }

                writer.write(std::array::from_ref(&(len as u8)))?;
                writer.write(std::array::from_ref(&byte))?;

                curr = byte;
                len = 1;
            }

            if len > 0 {
                writer.write(std::array::from_ref(&(len as u8)))?;
                writer.write(std::array::from_ref(&curr))?;
            }
        }

        Ok(output)
    }

    pub fn decode(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        let mut output = Vec::new();
        let mut reader = Cursor::new(bytes);

        let mut chunk_count_bytes = [0u8; 8];
        reader.read(&mut chunk_count_bytes)?;
        let chunk_count = usize::from_be_bytes(chunk_count_bytes);

        for _ in 0..chunk_count {
            let mut chunk = Vec::new();

            let mut index_bytes = [0u8; 8];
            reader.read(&mut index_bytes)?;
            let index = usize::from_be_bytes(index_bytes);

            loop {
                let mut len = 0u8;
                let mut byte = 0u8;

                if reader.read(std::array::from_mut(&mut len))? == 0 {
                    break;
                }

                if reader.read(std::array::from_mut(&mut byte))? == 0 {
                    break;
                }

                for _ in 0..len {
                    chunk.push(byte);
                }

                if chunk.len() == CHUNK_SIZE {
                    break;
                }
            }

            let data = crate::bwt::ibwt(&chunk, index);
            output.extend(data);
        }

        Ok(output)
    }
}
