use std::path::PathBuf;
use std::fs::{self};
use std::io::{self, Read, Write};

extern crate bit_vec;
extern crate rusqlite;
extern crate mime;
extern crate mime_guess;
#[macro_use]
extern crate error_chain;

use bit_vec::BitVec;


extern crate algos;

use algos::compression::{Compression, Decompression};
use algos::compression::rle::*;
use algos::compression::huffman::*;
use algos::compression::shannon::*;

use algos::coding::{Coding, Stats};
use algos::coding::repetition3::Repetition3;
use algos::coding::repetition5::Repetition5;

use algos::noise::NoiseLevel;

mod db;

type Error = &'static str;
type Result<T> = std::result::Result<T, Error>;

fn main() {
    run().unwrap();
}

fn run() -> Result<()> {
    db::create_schema().map_err(|_| "schema initialization failed")?;

    let data_dir = PathBuf::from("./data/files/");

    for entry in files(&data_dir)? {
        let db_file = db::File::for_dir_entry(&entry).map_err(|_| "io error")?;
        db_file.save().map_err(|_| "sqlite")?;

        let path = entry.path();
        println!("path: {}", path.to_str().ok_or("path")?);

        let mut file = fs::File::open(path).map_err(|_| "open file")?;

        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).map_err(|_| "read file")?;

        let original = BitVec::from_bytes(&content);
        println!("   original: {:?}", &original);

        let compressed = Rle.compress(&content).map_err(|_| "compression")?;
        println!(" compressed: {:?}", &compressed);

        // R5

        let encoded = Repetition5.encode(compressed.clone());
        println!("    encoded: {:?}", &encoded);
        println!();

        for noise in [NoiseLevel::Noise001, NoiseLevel::Noise005, NoiseLevel::Noise015].iter() {
            println!("      nioSe: {:?}", noise);

            let fucked_up: BitVec = noise.apply(encoded.iter()).collect();
            println!("  fucked up: {:?}", &fucked_up);

            let (decoded, stats) = Repetition5.decode(fucked_up);
            println!("    decoded: {:?}", &decoded);

            println!("    det/cor: {}/{}", stats.detected, stats.corrected);

            let dist = distance(&compressed, &decoded);
            println!("   distance: {}", dist);

            let coding = db::Coding {
                file_name: db_file.file_name.clone(),
                coding_name: "r5".into(),
                noise_rate: noise.to_str().to_string(),
                redundancy_rate: encoded.len() as f64 / original.len() as f64,
                size_decoded: original.len() as i32,
                size_encoded: encoded.len() as i32,
                corrected: stats.corrected as i32,
                detected: stats.detected as i32,
                not_corrected: dist as i32,
            };
            coding.save().map_err(|_| "save coding")?;

            // end R5

            let decompressed = Rle.decompress(decoded);
            match decompressed {
                Ok(ref vec) => {
                    let decompressed = BitVec::from_bytes(&vec);
                    println!("decompressed {:?}", &decompressed);

                    let error = distance(&original, &decompressed);
                    println!(" undetected: {}", error);

                    println!("result:");
                    std::io::stdout().write(&vec).unwrap();
                    std::io::stdout().write(b"\n\n").unwrap();
                    std::io::stdout().flush().unwrap();
                }
                Err(ref e) => println!("decompress error: {:?}", e),
            }
        }
    }
    Ok(())
}

pub fn distance(lhs: &BitVec, rhs: &BitVec) -> usize {
    let mut distance = 0u32;
    for (l, r) in lhs.blocks().zip(rhs.blocks()) {
        let l: u32 = l;
        let r: u32 = r;

        distance += (l ^ r).count_ones();
    }

    distance as usize
}

pub fn files(data_dir: &PathBuf) -> Result<Vec<fs::DirEntry>> {
    let vec = fs::read_dir(data_dir)
        .map_err(|_| "unable to open contacts file")?
        .filter_map(::std::result::Result::ok)
        .filter(|file| {
            file.file_type()
                .map(|ty| ty.is_file())
                .unwrap_or(false)
        })
        .collect();
    Ok(vec)
}