use super::{Coding, Stats};

use bit_vec::BitVec;

/// Repetition3 code.
#[derive(Debug)]
pub struct Repetition3;

pub const N: usize = 3;

impl Coding for Repetition3 {
    fn encode(&self, input: BitVec) -> BitVec {
        let result_len = input.len() * 3;
        let mut result = BitVec::from_elem(result_len, false);
        for i in 0..(input.len()) {
            if let Some(bit) = input.get(i) {
                let j = i * 3;
                result.set(j, bit);
                result.set(j + 1, bit);
                result.set(j + 2, bit);
            }
        }
        result
    }

    fn decode(&self, input: BitVec) -> (BitVec, Stats) {
        let mut stats = Stats::new();

        let result_len = input.len() / 3;
        let mut result = BitVec::from_elem(result_len, false);
        let mut j = 0;

        for i in 0..result_len {
            let mut count = 0;

            while j / N < i + 1 {
                if let Some(bit) = input.get(j) {
                    if bit {
                        count += 1;
                    }
                }
                j += 1;
            }
            if count >= 2 {
                result.set(i, true);
            }
            if !(count == 0 || count == N) {
                stats.corrected += 1;
                stats.detected += 1;
            }
        }
        (result, stats)
    }
}