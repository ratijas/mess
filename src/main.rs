#[macro_use]
extern crate lazy_static;
extern crate bit_vec;
extern crate rusqlite;
extern crate mime;
extern crate mime_guess;
extern crate threadpool;
extern crate num_cpus;
extern crate algos;

use std::path::PathBuf;
use std::fs::{self};
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};

use bit_vec::BitVec;
use threadpool::ThreadPool;

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

type Result<T> = std::result::Result<T, Error>;

fn main() {
    run().unwrap();
}

fn run() -> Result<()> {
    db::create_schema()?;

    let data_dir = PathBuf::from("./data/files/");

    let n_workers = num_cpus::get();
    let pool = ThreadPool::new(n_workers);

    for entry in files(&data_dir)? {
        let db_file = db::File::for_dir_entry(&entry)?;
        db_file.save()?;

        let path = entry.path();

        // println!("path: {}", path.to_str().ok_or("path")?);

        let mut file = fs::File::open(path)?;
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content)?;
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
    let vec = fs::read_dir(data_dir)?
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

    let (compressed, time_compress) = profile(|| {
        Ok(compression.compress(original)?)
    })?;

    // println!(" compressed: {:?}", &compressed);

    let mut c = db::Compression {
        file_name: db_file.file_name.clone(),
        compression: compression_name.into(),
        compress_rate: compressed.len() as f64 / (8 * original.len()) as f64,
        size_compressed: compressed.len() as i64,
        time_compress,
        time_decompress: None,
    };
    c.save()?;

    for &(coding, coding_name) in [
        (&Repetition3 as &Coding, "r3"),
        (&Repetition5, "r5"),
        (&Parity, "parity"),
        (&Hamming, "hamming")
    ].iter() {
        // println!("    coding: {}", coding_name);

        let (encoded, time_encode) = profile(|| {
            Ok(coding.encode(compressed.clone()))
        })?;

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

            let ((decoded, stats), time_decode) = profile(move || {
                Ok(coding.decode(fucked_up))
            })?;

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
                time_encode,
                time_decode,
            };
            coding.save()?;

            match profile(|| Ok(compression.decompress(decoded)?)) {
                Ok((_decompressed, time_decompress)) => {
                    if c.time_decompress == None {
                        c.time_decompress = Some(time_decompress);
                        c.save()?;
                    }
                }
                Err(_) => {}
            }

            /*
            match decompressed {
                Ok(_) => {
                    // let decompressed = BitVec::from_bytes(&vec);
                    // // println!("decompressed {:?}", &decompressed);

                    // let error = distance_vec(&original, &vec);
                    // println!(" undetected: {}", error);

                    // println!("result:");
                    // std::io::stdout().write(&vec).unwrap();
                }
                Err(_) => {} // println!("decompress error: {:?}", e),
            }
            */

            std::io::stdout().write(b".")?;
            std::io::stdout().flush()?;
        }
    }
    std::io::stdout().write(b"\n")?;
    std::io::stdout().flush()?;
    Ok(())
}

fn elapsed_millis(instant: Instant) -> i64 {
    let elapsed: Duration = instant.elapsed();
    let millis = 1000 * elapsed.as_secs() as i64;
    millis + (elapsed.subsec_nanos() as i64 / 1_000_000_i64)
}

fn profile<F, T>(f: F) -> Result<(T, i64)>
    where F: FnOnce() -> Result<T>
{
    let instant = Instant::now();
    let result = f()?;
    Ok((result, elapsed_millis(instant)))
}

// error chain
#[derive(Debug)]
pub enum Error {
    Sql(rusqlite::Error),
    Str(&'static str),
    Compression(algos::compression::Error),
    Io(std::io::Error),
    OsString(std::ffi::OsString),
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::Sql(e)
    }
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::Str(s)
    }
}

impl From<algos::compression::Error> for Error {
    fn from(e: algos::compression::Error) -> Self {
        Error::Compression(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<std::ffi::OsString> for Error {
    fn from(s: std::ffi::OsString) -> Self {
        Error::OsString(s)
    }
}