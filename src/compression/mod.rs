//! Various compression methods.

pub mod rle;
pub mod huffman;
pub mod shannon;

use bit_vec::BitVec;

/// General compression trait.
///
/// A lossless compression's invariant requires implementing algorithms to be invertible,
/// i.e. `decompress(compress(input)) == input`.
///
/// Type `T` is the type of elements in the stream to be compressed / decompressed.
/// Probably, byte type `u8` is the most useful here.
pub trait Compression<T> {
    fn compress(&self, input: &[T]) -> Result<BitVec, Error>;

    fn decompress(&self, input: BitVec) -> Result<Vec<T>, Error>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    ExpectedMoreData,
    ZeroRepetition,
}