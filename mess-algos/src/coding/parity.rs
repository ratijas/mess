use super::{Coding, Stats};

use bit_vec::BitVec;

/// Parity-check code.
#[derive(Debug)]
pub struct Parity;

impl Coding for Parity {
    fn encode(&self, input: BitVec) -> BitVec {
        let len = input.len();
        let result_len = 5 * len / 4 + (if len % 4 > 0 { len % 4 + 1 } else { 0 });
        let mut result = BitVec::from_elem(result_len, false);
        let mut j = 0;
        let mut p = false;

        for i in 0..len {
            if let Some(bit) = input.get(i) {
                result.set(j, bit);
                j += 1;
                p = p ^ bit;

                if i % 4 == 3 {
                    result.set(j, p);
                    p = false;
                    j += 1;
                }
            }
        }
        result
    }

    fn decode(&self, input: BitVec) -> (BitVec, Stats) {
        let mut stats = Stats::new();

        let len = input.len();
        let result_len = 4 * len / 5 + (if len % 5 > 0 { len % 5 - 1 } else { 0 });
        let mut result = BitVec::from_elem(result_len, false);

        let mut j = 0;
        let mut p = false;  // p for parity

        for i in 0..result_len {
            if let Some(bit) = input.get(j) {
                result.set(i, bit);
                j += 1;
                p = p ^ bit;

                // if j is parity bit index
                if j % 5 == 4 {
                    if let Some(parity) = input.get(j) {
                        if p != parity {
                            stats.detected += 1;
                        }
                    }
                    p = false;  // reset parity
                    j += 1;
                }
            }
        }
        (result, stats)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(5, Parity.encode(BitVec::from_elem(4, false)).len());
        assert_eq!(10, Parity.encode(BitVec::from_bytes(&[0b01101001u8])).len());
    }
}