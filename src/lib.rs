extern crate bit_vec;

pub mod coding;
pub mod compression;

pub use coding::{Coding, Stats};
pub use compression::Compression;
