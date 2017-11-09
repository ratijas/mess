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
pub trait Compression<'a, T: 'a> {
    type Error;

    fn compress<I>(&self, input: I) -> Result<BitVec, Self::Error>
        where I: IntoIterator<Item=T>;

    fn decompress(&self, input: BitVec) -> Result<Vec<T>, Self::Error>;
}
