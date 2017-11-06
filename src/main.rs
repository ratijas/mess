extern crate bit_vec;

//use std::ops::Neg;

use std::collections::HashMap;
use std::hash::Hash;

use bit_vec::BitVec;

pub mod rle;
pub mod huffman;
//pub mod parity;
pub mod repetition3;
pub mod repetition5;

//use rle::*;
//use parity::*;
//use repetition3::*;
use repetition5::*;
//use huffman::*;

fn main() {
    let mut code = BitVec::from_bytes(&[8u8,6u8]);
    println!("{:?}", code);
    code= Repetition5.encode(BitVec::from_bytes(&[8u8,6u8]));
    println!("{:?}", code);
    code.set(7,true);
    code.set(8,true);
    println!("{:?}", code);
    let (code, _vec) = Repetition5.decode(code);
    println!("{:?}", code);


    /*let comp = Rle.compress(b"hello, world!");
    println!("rle compressed: {:?}", &comp);

    println!("{}", 0b01111111u8);
    println!("u8: {}, i8: {}", std::u8::MAX, std::i8::MAX);
    println!("u8 as i8: {}, {}", 5u8 as i8, 203u8 as i8);
    println!("i8 as u8: {}", (-26i8) as u8);
    let buffer = [0u8; (-(std::i8::MIN as isize)) as usize];
    println!("buffer len");
    println!("{}: buffer.len()", buffer.len());
    println!("{}: buffer.len() as isize", buffer.len() as isize);
    println!("{}: (buffer.len() as isize).neg()", (buffer.len() as isize).neg());
    println!("{}: (buffer.len() as isize).neg() as i8", (buffer.len() as isize).neg() as i8);
    println!("{}: (buffer.len() as isize).neg() as i8 as u8", (buffer.len() as isize).neg() as i8 as u8);
    println!("{}: (buffer.len() as isize).neg() as u8", (buffer.len() as isize).neg() as u8);
    println!("{}: (buffer.len() as isize).neg() as u8 as i8", (buffer.len() as isize).neg() as u8 as i8);
    println!("{}: (((buffer.len() as isize).neg() as u8 as i8) as isize).neg()", (((buffer.len() as isize).neg() as u8 as i8) as isize).neg());

    //    let orig = Rle.decompress(&comp).expect("error while decompressing");
    //    println!("original: {}", orig);

    let i = 5i64;

    match i {
        0 => println!("zero"),
        1 => println!("one"),
        x if (x >= 2 && x <= 9) => {
            println!("one-digit");
        }
        _ => println!("other"),
    }*/
}

pub fn dictionary_compression<'a, T, I>(dict: HashMap<T, BitVec>, input: I) -> Result<BitVec, ()>
    where
        T: Eq + Hash + 'a,
        I: IntoIterator<Item=&'a T> + 'a,
{
    let mut output = BitVec::new();
    for i in input.into_iter() {
        match dict.get(i) {
            Some(code) => output.extend(code),
            None => return Err(()),
        }
    }
    Ok(output)
}

/// General compressor trait.
///
/// A lossless compression's invariant requires implementing algorithms to be invertible,
/// i.e. `decompress(compress(input)) == input`.
///
/// Type `T` is the type of elements in the stream to be compressed / decompressed.
/// Probably, byte type `u8` is the most useful here.
pub trait Compressor<'a, T: 'a> {
    type Error;
    fn compress<I>(&self, input: I) -> BitVec
        where I: IntoIterator<Item=&'a T> + 'a;
    fn decompress(&self, input: BitVec) -> Result<Vec<T>, Self::Error>;
}

pub trait Code {
    fn encode(&self, input: BitVec) -> BitVec;
    fn decode (&self, input: BitVec) -> (BitVec, Vec<usize>);
}

/*
pub enum CodingResult {
    Ok(Vec<u8>),
    Err(),
}

pub struct Stats {
    _corrections: usize,
}

pub trait Coding {
    fn encode();
    fn decode(&mut self, input: &[u8]) -> CodingResult;

    /// flip bit at `position` and update coder's statistics.
    fn correct(&mut self, position: usize);
    /// report an unrecoverable error.
    fn error(&mut self, position: usize);

    fn stats(&self) -> &Stats;
    fn stats_mut(&mut self) -> &mut Stats;
}
*/