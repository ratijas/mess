use super::{Coding, Stats};

use bit_vec::BitVec;

/// Repetition5 code.
#[derive(Debug)]
pub struct Repetition5;

pub const N: usize = 5;

impl Coding for Repetition5 {
    fn encode(&self, input: BitVec) -> BitVec {
        let len = input.iter().count();
        let result_len = len * 5;
        let mut result = BitVec::from_elem(result_len, false);
        for i in 0..len {
            if let Some(bit) = input.get(i) {
                let j = i * 5;
                result.set(j, bit);
                result.set(j + 1, bit);
                result.set(j + 2, bit);
                result.set(j + 3, bit);
                result.set(j + 4, bit);
            }
        }
        result
    }

    fn decode(&self, input: BitVec) -> (BitVec, Stats) {
        let mut stats = Stats::new();

        let result_len = input.len() / 5;
        let mut result = BitVec::from_elem(result_len, false);
        let mut j = 0;

        for i in 0..result_len {
            let mut count = 0;

            while j / 5 < i + 1 {
                if let Some(bit) = input.get(j) {
                    if bit {
                        count += 1;
                    }
                }
                j += 1;
            }
            if count >= 3 {
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


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut code = BitVec::from_bytes(&[8u8, 6u8]);
        println!("{:?}", code);
        code = Repetition5.encode(BitVec::from_bytes(&[8u8, 6u8]));
        println!("{:?}", code);
        code.set(7, true);
        code.set(8, true);
        println!("{:?}", code);
        let (code, _vec) = Repetition5.decode(code);
        println!("{:?}", code);
    }
}