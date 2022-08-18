extern crate clap;
use error_chain::error_chain;
use std::{fs::{File}};
use serde::{Deserialize};
use base64::{decode_config, URL_SAFE};
use std::io::Write;
use std::str;
use clap::{Arg, App};

#[derive(Deserialize)]
struct OffsetResponse {
    offset: String,
    size: String,
}

#[derive(Deserialize)]
struct ChunkResponse {
    chunk: String,
}

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

async fn get_offset(transaction_id: String) -> (i64, i64) {
    let offset_res = reqwest::get(format!("https://arweave.net/tx/{transaction_id}/offset")).await.unwrap();
    let offset_body = offset_res.json::<OffsetResponse>().await.unwrap();

    return (offset_body.offset.parse().unwrap(), offset_body.size.parse().unwrap());
}

async fn get_chunk(offset: i64) -> Vec<u8> {
    let chunk_url = format!("https://arweave.net/chunk/{offset}");
    let chunk_res = reqwest::get(chunk_url).await.unwrap();
    let chunk_body = chunk_res.json::<ChunkResponse>().await.unwrap();
    return decode_config(chunk_body.chunk, URL_SAFE).unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    let chunk_size: i64 = 256 * 1024;

    // Parse args
    let matches = App::new("Arweave Downloader")
            .version("0.0.1")
            .author("Tuguldur Baigalmaa <philoniare@gmail.com>")
            .about("Simple CLI tool to download ArWeave transaction data in chunks")
            .arg(Arg::with_name("TRANSACTION")
                .short("t")
                .long("transaction")
                .required(true)
                .takes_value(true)
                .help("Transaction ID"))
            .arg(Arg::with_name("OUTPUT")
                .short("o")
                .long("output")
                .required(true)
                .takes_value(true)
                .help("name of the output file"))
            .get_matches();
    let transaction_id = matches.value_of("TRANSACTION").unwrap();
    let output_filename = matches.value_of("OUTPUT").unwrap();

    // Fetch transaction offset and size
    let (_offset, _size) = get_offset(transaction_id.to_string()).await;

    // Download individual chunks and append
    let mut contents: Vec<u8> = vec![];
    let output_file = File::create(output_filename);

    let mut _current_offset = _offset;
    let mut _remaining_size = _size;

    while _remaining_size > 0 {
        let mut chunk_bytes = get_chunk(_current_offset).await;
        chunk_bytes.append(&mut contents);
        contents = chunk_bytes;
        _remaining_size -= chunk_size;
        _current_offset -= chunk_size;
    }

    // Write the final output to the specified file
    output_file.unwrap().write_all(&contents).expect("Unable to write to file");
    println!("Done downloading the file to {output_filename}");

    Ok(())
}