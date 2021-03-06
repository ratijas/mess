pub mod parity;
pub mod repetition3;
pub mod repetition5;
pub mod hamming;

use bit_vec::BitVec;


#[derive(Clone, Debug)]
pub struct Stats {
    pub detected: u32,
    pub corrected: u32,
}

impl Stats {
    pub fn new() -> Stats {
        Stats { detected: 0, corrected: 0 }
    }
}


pub trait Coding {
    fn encode(&self, input: BitVec) -> BitVec;
    fn decode(&self, input: BitVec) -> (BitVec, Stats);
}
