use anyhow::bail;
use clap::Parser;

use markov_huffman::huffman::HuffmanCoder;

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

    let coder = HuffmanCoder::new();

    let output = if args.compress {
        coder.encode(&input)?
    } else {
        coder.decode(&input)?
    };

    std::fs::write(&args.output, output)?;

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
}
