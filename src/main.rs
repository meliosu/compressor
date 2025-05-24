use anyhow::bail;
use clap::Parser;

use markov_huffman::{
    bwt_coder::BWTCoder, bwt_huffman::BWTHuffmanCoder, bwt_mtf_rle_huffman::BwtMtfRleHuffmanCoder,
    huffman::HuffmanCoder, markov_arithmetic::MarkovArithmeticCoder, rans::ANSCoder,
};

fn main() {
    if let Err(e) = app() {
        eprintln!("{e}");
    }
}

fn app() -> anyhow::Result<()> {
    let args = Args::parse();

    if (args.compress && args.decompress) || (!args.compress && !args.decompress) {
        bail!("Select one of --compress or --decompress");
    }

    let input = std::fs::read(&args.input)?;

    match args.algorithm.as_str() {
        "markov-huffman" => {
            let coder = HuffmanCoder::new();

            let output = if args.compress {
                coder.encode(&input)?
            } else {
                coder.decode(&input)?
            };

            std::fs::write(&args.output, output)?;
        }

        "bwt" => {
            let coder = BWTCoder::new();

            let output = if args.compress {
                coder.encode(&input)?
            } else {
                coder.decode(&input)?
            };

            std::fs::write(&args.output, output)?;
        }

        "bwt-huffman" => {
            let coder = BWTHuffmanCoder::new();

            let output = if args.compress {
                coder.encode(&input)?
            } else {
                coder.decode(&input)?
            };

            std::fs::write(&args.output, output)?;
        }

        "markov-arithmetic" => {
            let coder = MarkovArithmeticCoder::new();

            let output = if args.compress {
                coder.encode(&input)?
            } else {
                coder.decode(&input)?
            };

            std::fs::write(&args.output, output)?;
        }

        "bwt-mtf-rle-huffman" => {
            let coder = BwtMtfRleHuffmanCoder::new();

            let output = if args.compress {
                coder.encode(&input)?
            } else {
                coder.decode(&input)?
            };

            std::fs::write(&args.output, output)?;
        }

        "ans" => {
            let mut coder = ANSCoder::new();

            let output = if args.compress {
                coder.encode(&input)?
            } else {
                coder.decode(&input)?
            };

            std::fs::write(&args.output, output)?;
        }

        _ => bail!("Unknown algorithm"),
    }

    Ok(())
}

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    compress: bool,

    #[arg(short, long)]
    decompress: bool,

    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long)]
    algorithm: String,
}
