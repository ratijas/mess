use std::path::PathBuf;
use std::fs::{self};
use std::io::{Read, Write};
use std::sync::Arc;

extern crate bit_vec;
extern crate rusqlite;
extern crate mime;
extern crate mime_guess;
extern crate threadpool;
extern crate num_cpus;

use bit_vec::BitVec;
use threadpool::ThreadPool;


extern crate algos;

use algos::compression::Compression;
use algos::compression::rle::*;
use algos::compression::huffman::*;
use algos::compression::shannon::*;

use algos::coding::Coding;
use algos::coding::repetition3::Repetition3;
use algos::coding::repetition5::Repetition5;
use algos::coding::parity::Parity;
use algos::coding::hamming::Hamming;

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

    let n_workers = num_cpus::get();
    let pool = ThreadPool::new(n_workers);

    for entry in files(&data_dir)?.into_iter().filter(|f| f.path().as_os_str().to_str().unwrap() == "./data/files/Information Theory Project 2 Datasets /WAV/04_The_Devil_In_I.wav") {
        let db_file = db::File::for_dir_entry(&entry).map_err(|_| "io error")?;
        db_file.save().map_err(|_| "sqlite")?;

        let path = entry.path();
        println!("path: {}", path.to_str().ok_or("path")?);

        let mut file = fs::File::open(path).map_err(|_| "open file")?;
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content).map_err(|_| "read file")?;
        let content = Arc::new(content);  // non-mutable shared

        {
            let db_file = db_file.clone();
            let content = Arc::clone(&content);
            pool.execute(move || {
                let compression = Huffman::<u8>::optimal_for(&*content);
                live_with_it("huffman", &compression, &*content, &db_file).unwrap();
            });
        }
        {
            let db_file = db_file.clone();
            let content = Arc::clone(&content);
            pool.execute(move || {
                let compression = ShannonFano::<u8>::optimal_for(&*content);
                live_with_it("shannon", &compression, &*content, &db_file).unwrap();
            });
        }
        {
            let db_file = db_file.clone();
            let content = Arc::clone(&content);
            pool.execute(move || {
                let compression = Rle;
                live_with_it("rle", &compression, &*content, &db_file).unwrap();
            });
        }
    }
    pool.join();
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

pub fn distance_vec(lhs: &[u8], rhs: &[u8]) -> usize {
    let mut distance = 0u32;
    for (l, r) in lhs.iter().zip(rhs.iter()) {
        let l: u8 = *l;
        let r: u8 = *r;

        distance += (l ^ r).count_ones();
    }

    distance as usize
}

pub fn files(data_dir: &PathBuf) -> Result<Vec<fs::DirEntry>> {
    let vec = fs::read_dir(data_dir)
        .map_err(|_| "unable to open contacts file")?
        .filter_map(::std::result::Result::ok)
        .flat_map(|entry| {
            let ty = entry.file_type().unwrap();
            if ty.is_file() {
                vec![entry]
            } else if ty.is_dir() {
                files(&entry.path()).unwrap()
            } else {
                vec![]
            }
        })
        .filter(|file| {
            file.file_type()
                .map(|ty| ty.is_file())
                .unwrap_or(false)
        })
        .collect();
    Ok(vec)
}

pub fn live_with_it(compression_name: &str,
                    compression: &Compression<u8>,
                    original: &[u8],
                    db_file: &db::File) -> Result<()>
{
    // println!("compression: {}", compression_name);

    let compressed = compression.compress(original).map_err(|_| "compression")?;
    // println!(" compressed: {:?}", &compressed);

    let c = db::Compression {
        file_name: db_file.file_name.clone(),
        compression: compression_name.into(),
        compress_rate: compressed.len() as f64 / (8 * original.len()) as f64,
        size_compressed: compressed.len() as i64,
    };
    c.save().map_err(|_| "save compression")?;

    for &(coding, coding_name) in [
        (&Repetition3 as &Coding, "r3"),
        (&Repetition5, "r5"),
        (&Parity, "parity"),
        (&Hamming, "hamming")
    ].iter() {
        println!("    coding: {}", coding_name);

        let encoded = coding.encode(compressed.clone());
        // println!("    encoded: {:?}", &encoded);

        let redundancy_rate = encoded.len() as f64 / (compressed.len()) as f64;

        for noise in [
            NoiseLevel::Noise001,
            NoiseLevel::Noise005,
            NoiseLevel::Noise015
        ].iter() {
            // println!("      nioSe: {:?}", noise);

            let fucked_up: BitVec = noise.apply(encoded.iter()).collect();
            // println!("  fucked up: {:?}", &fucked_up);

            let (decoded, stats) = coding.decode(fucked_up);
            // println!("    decoded: {:?}", &decoded);

            // println!("    det/cor: {}/{}", stats.detected, stats.corrected);

            let dist = distance(&compressed, &decoded);
            // println!("   distance: {}", dist);

            let coding = db::Coding {
                file_name: db_file.file_name.clone(),
                compression: compression_name.into(),
                coding_name: coding_name.into(),
                noise_rate: noise.to_str().to_string(),
                redundancy_rate,
                size_decoded: compressed.len() as i64,
                size_encoded: encoded.len() as i64,
                corrected: stats.corrected as i64,
                detected: stats.detected as i64,
                not_corrected: dist as i64,
            };
            coding.save().map_err(|_| "save coding")?;

            let decompressed = Rle.decompress(decoded);
            match decompressed {
                Ok(ref vec) => {
                    // let decompressed = BitVec::from_bytes(&vec);
                    // // println!("decompressed {:?}", &decompressed);

                    let error = distance_vec(&original, &vec);
                    println!(" undetected: {}", error);

                    // println!("result:");
                    // std::io::stdout().write(&vec).unwrap();
                    // std::io::stdout().write(b"\n\n").unwrap();
                    // std::io::stdout().flush().unwrap();
                }
                Err(ref e) => println!("decompress error: {:?}", e),
            }
        }
    }
    Ok(())
}