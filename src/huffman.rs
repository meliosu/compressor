#![allow(unused)]

use std::collections::VecDeque;
use std::io;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use std::marker::PhantomData;
use std::process::Output;

use bitbit::BitReader;
use bitbit::BitWriter;
use bitbit::MSB;

pub struct TreeNode {
    frequency: u64,
    kind: TreeNodeKind,
}

pub enum TreeNodeKind {
    Leaf {
        byte: u8,
    },
    Node {
        left: Box<TreeNode>,
        right: Box<TreeNode>,
    },
}

#[derive(Clone, Copy, Default)]
pub struct Code {
    word: u64,
    len: usize,
}

impl TreeNode {
    pub fn build(frequencies: &[u64; 256]) -> Option<Self> {
        let mut queue = VecDeque::<TreeNode>::new();

        for byte in 0..=255 {
            let frequency = frequencies[byte as usize];
            let index = queue
                .binary_search_by_key(&frequency, |node| node.frequency)
                .unwrap_or_else(|idx| idx);

            queue.insert(
                index,
                TreeNode {
                    frequency,
                    kind: TreeNodeKind::Leaf { byte },
                },
            );
        }

        while queue.len() > 1 {
            let left = queue.pop_front().unwrap();
            let right = queue.pop_front().unwrap();

            let frequency = left.frequency + right.frequency;

            let index = queue
                .binary_search_by_key(&frequency, |node| node.frequency)
                .unwrap_or_else(|idx| idx);

            queue.insert(
                index,
                TreeNode {
                    frequency,
                    kind: TreeNodeKind::Node {
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                },
            );
        }

        queue.pop_front()
    }

    pub fn codes(&self) -> [Code; 256] {
        fn codes_recursive(node: &TreeNode, codes: &mut [Code; 256], word: u64, len: usize) {
            match &node.kind {
                TreeNodeKind::Leaf { byte } => codes[*byte as usize] = Code { word, len },
                TreeNodeKind::Node { left, right } => {
                    codes_recursive(&left, codes, word << 1, len + 1);
                    codes_recursive(&right, codes, (word << 1) | 1, len + 1);
                }
            }
        }

        let mut codes = [Code::default(); 256];
        codes_recursive(self, &mut codes, 0, 0);
        codes
    }

    pub fn encode<W: Write>(&self, writer: &mut BitWriter<W>) -> io::Result<()> {
        match &self.kind {
            TreeNodeKind::Leaf { byte } => {
                writer.write_bit(true)?;
                writer.write_byte(*byte)?;
            }

            TreeNodeKind::Node { left, right } => {
                writer.write_bit(false)?;
                left.encode(writer)?;
                right.encode(writer)?;
            }
        }

        Ok(())
    }

    pub fn decode<R: Read>(reader: &mut BitReader<R, MSB>) -> io::Result<Self> {
        if reader.read_bit()? {
            let byte = reader.read_byte()?;

            Ok(TreeNode {
                frequency: 0,
                kind: TreeNodeKind::Leaf { byte },
            })
        } else {
            let left = TreeNode::decode(reader)?;
            let right = TreeNode::decode(reader)?;

            Ok(TreeNode {
                frequency: 0,
                kind: TreeNodeKind::Node {
                    left: Box::new(left),
                    right: Box::new(right),
                },
            })
        }
    }

    pub fn decode_symbol<R: Read>(&self, reader: &mut BitReader<R, MSB>) -> io::Result<u8> {
        let mut node = self;

        loop {
            match &node.kind {
                TreeNodeKind::Leaf { byte } => return Ok(*byte),
                TreeNodeKind::Node { left, right } => {
                    if reader.read_bit()? {
                        node = right;
                    } else {
                        node = left;
                    }
                }
            }
        }
    }
}

pub struct HuffmanCoder {
    p: PhantomData<()>,
}

impl HuffmanCoder {
    pub fn new() -> Self {
        HuffmanCoder { p: PhantomData }
    }

    pub fn encode(&self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        let mut output = Vec::new();
        let mut output_cursor = Cursor::new(&mut output);

        let mut length_bytes = bytes.len().to_be_bytes();
        output_cursor.write(&length_bytes)?;

        let mut writer = BitWriter::new(output_cursor);

        let tables = self.build_frequency_tables(bytes);
        let trees = tables.map(|frequencies| TreeNode::build(&frequencies).unwrap());

        for tree in &trees {
            tree.encode(&mut writer)?;
        }

        let codes = trees.map(|tree| tree.codes());

        let mut previous = 0u8;

        for &byte in bytes {
            let code = codes[previous as usize][byte as usize];
            writer.write_bits(code.word as u32, code.len)?;
            previous = byte;
        }

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

        let mut trees = [const {
            TreeNode {
                frequency: 0,
                kind: TreeNodeKind::Leaf { byte: 0 },
            }
        }; 256];

        for i in 0..256 {
            trees[i] = TreeNode::decode(&mut reader)?;
        }

        let mut previous = 0u8;

        for _ in 0..length {
            let byte = trees[previous as usize].decode_symbol(&mut reader)?;
            output.push(byte);
            previous = byte;
        }

        Ok(output)
    }

    fn build_frequency_tables(&self, bytes: &[u8]) -> [[u64; 256]; 256] {
        let mut previous = 0u8;
        let mut frequencies = [[0u64; 256]; 256];

        for &byte in bytes {
            frequencies[previous as usize][byte as usize] += 1;
            previous = byte;
        }

        frequencies
    }
}

impl Code {
    pub fn encode<W: Write>(&self, writer: &mut BitWriter<W>) -> io::Result<()> {
        writer.write_bits(self.word as u32, self.len)
    }
}
