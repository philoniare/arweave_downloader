use error_chain::error_chain;
use std::{fs::{File}};
use serde::{Deserialize};
use base64::{decode_config, URL_SAFE};
use std::io::Write; // bring trait into scope
use std::str;


extern crate clap;

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


#[tokio::main]
async fn main() -> Result<()> {
    // Parse args
    let matches = App::new("Arweave Downloader")
            .version("0.0.1")
            .author("Tuguldur Baigalmaa <philoniare@gmail.com>")
            .about("Simple CLI tool to download ArWeave transaction data in chunks")
            .arg(Arg::with_name("TRANSACTION")
                .short("t")
                .long("transaction")
                .required(true)
                .index(1)
                .takes_value(true)
                .help("Transaction ID"))
            .arg(Arg::with_name("OUTPUT")
                .short("o")
                .long("output")
                .required(true)
                .index(2)
                .takes_value(true)
                .help("name of the output file"))
            .get_matches();
    let transaction_id = matches.value_of("TRANSACTION").unwrap();

    let output_filename = matches.value_of("OUTPUT").unwrap();


    // Fetch transaction offset and size
    let offset_res = reqwest::get(format!("https://arweave.net/tx/{transaction_id}/offset")).await?;
    let offset_body = offset_res.json::<OffsetResponse>().await?;

    let _offset: i32 = offset_body.offset.parse().unwrap();
    let _size: i32 = offset_body.size.parse().unwrap();

    // Download individual chunks
    let mut contents = String::new();
    let output_file = File::create(output_filename);

    for n in 0.._size {
        let _current_offset = _offset - n;
        let chunk_url = format!("https://arweave.net/chunk/{_current_offset}");
        let chunk_res = reqwest::get(chunk_url).await?;
        let chunk_body = chunk_res.json::<ChunkResponse>().await?;
        let chunk_bytes = decode_config(chunk_body.chunk, URL_SAFE).unwrap();
        let chunk_string: String = String::from_utf8(chunk_bytes.clone()).unwrap();
        contents.push_str(&chunk_string);
    }
    output_file.unwrap().write_all(contents.as_bytes()).expect("Unable to write to file");

    println!("Done downloading the file to {output_filename}");

    Ok(())
}