// bits
extern crate bit_vec;

// network
extern crate reqwest;

// serde
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[allow(unused)]
#[macro_use]
extern crate serde_json;
extern crate base64;


pub mod coding;
pub mod compression;

pub use coding::{Coding, Stats};
pub use compression::Compression;


pub mod types;
pub mod methods;