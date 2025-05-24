#![allow(unused)]

use std::{
    io::{self, Cursor, Read, Write},
    marker::PhantomData,
    usize,
};

use arcode::{ArithmeticDecoder, ArithmeticEncoder, Model};
use bitbit::{BitReader, BitWriter, MSB};

pub struct MarkovArithmeticCoder {
    p: PhantomData<()>,
}

impl MarkovArithmeticCoder {
    pub fn new() -> Self {
        MarkovArithmeticCoder { p: PhantomData }
    }

    pub fn encode(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        let mut output = Vec::new();
        let mut output_cursor = Cursor::new(&mut output);

        output_cursor.write(&bytes.len().to_be_bytes())?;

        let mut writer = BitWriter::new(output_cursor);

        let mut models = (0..256)
            .map(|_| Model::builder().num_symbols(256).build())
            .collect::<Vec<_>>();

        let mut coders = (0..256)
            .map(|_| ArithmeticEncoder::new(48))
            .collect::<Vec<_>>();

        let mut previous = 0u8;

        for &byte in bytes {
            coders[previous as usize].encode(
                byte as u32,
                &models[previous as usize],
                &mut writer,
            )?;

            models[previous as usize].update_symbol(byte as u32);
            previous = byte;
        }

        coders[previous as usize].finish_encode(&mut writer)?;
        writer.pad_to_byte()?;

        Ok(output)
    }

    pub fn decode(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        let mut output = Vec::new();
        let mut input_cursor = Cursor::new(bytes);

        let mut length_bytes = [0u8; 8];
        input_cursor.read(&mut length_bytes)?;
        let length = usize::from_be_bytes(length_bytes);

        let mut reader = BitReader::<_, MSB>::new(input_cursor);

        let mut models = (0..256)
            .map(|_| Model::builder().num_symbols(256).build())
            .collect::<Vec<_>>();

        let mut coders = (0..256)
            .map(|_| ArithmeticDecoder::new(48))
            .collect::<Vec<_>>();

        let mut previous = 0u8;

        for _ in 0..length {
            let byte = coders[previous as usize].decode(&models[previous as usize], &mut reader)?;
            models[previous as usize].update_symbol(byte);
            output.push(byte as u8);
            previous = byte as u8;
        }

        Ok(output)
    }
}
