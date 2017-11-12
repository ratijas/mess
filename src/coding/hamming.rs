//! Hamming (4/7)


use super::{Coding, Stats};
use bit_vec::BitVec;

#[derive(Debug)]
pub struct Hamming;

impl Coding for Hamming {
    fn encode(&self, input: BitVec) -> BitVec {
        let len = input.len();
        let result_len = 7 * (len / 4) + if len % 4 == 1 { 2 } else if len % 4 > 1 { 3 } else { 0 } + len % 4;

        let mut result = BitVec::from_elem(result_len, false);

        for i in 0..(len + 3) / 4 {
            let b0 = input.get(0 + i * 4).unwrap();
            let b1 = if 1 + i * 4 < len { input.get(1 + i * 4).unwrap() } else { false };
            let b2 = if 2 + i * 4 < len { input.get(2 + i * 4).unwrap() } else { false };
            let b3 = if 3 + i * 4 < len { input.get(3 + i * 4).unwrap() } else { false };

            result.set(0 + i * 7, b0 ^ b1 ^ b3);//
            result.set(1 + i * 7, b0 ^ b2 ^ b3);//
            result.set(2 + i * 7, input.get(0 + i * 4).unwrap());
            if 3 + i * 7 < result_len { result.set(3 + i * 7, b1 ^ b2 ^ b3) };
            if 4 + i * 7 < result_len { result.set(4 + i * 7, b1) };
            if 5 + i * 7 < result_len { result.set(5 + i * 7, b2); }
            if 6 + i * 7 < result_len { result.set(6 + i * 7, b3); }
        }

        result
    }

    fn decode(&self, mut input: BitVec) -> (BitVec, Stats) {
        let mut stats = Stats::new();
        let len = input.iter().count();
        let result_len = len / 7 * 4 + match len % 7 {
            3 => 1,
            l if l > 3 => l - 3,
            _ => 0,
        };

        let mut result = BitVec::from_elem(result_len, false);
        let mut diff;
        for i in 0..(len + 6) / 7 {
            diff = 0;
            if input.get(2 + i * 7).unwrap() ^ input.get(4 + i * 7).unwrap() ^ input.get(6 + i * 7).unwrap() != input.get(0 + i * 7).unwrap() { diff += 1 };
            if input.get(2 + i * 7).unwrap() ^ input.get(5 + i * 7).unwrap() ^ input.get(6 + i * 7).unwrap() != input.get(1 + i * 7).unwrap() { diff += 2 };
            if input.get(4 + i * 7).unwrap() ^ input.get(5 + i * 7).unwrap() ^ input.get(6 + i * 7).unwrap() != input.get(3 + i * 7).unwrap() { diff += 4 };
            if diff > 0 {
                diff -= 1;
                let get = input.get(diff + i * 7).unwrap();
                input.set(diff + 7 * i, !get);
                stats.detected += 1;
                stats.corrected += 1;
            }
        }

        let mut j = 0;
        for i in 0..len {
            if i % 7 == 2 {
                result.set(j, input.get(i).unwrap());
                j += 1;
            }
            if i % 7 == 4 {
                result.set(j, input.get(i).unwrap());
                j += 1;
            }
            if i % 7 == 5 {
                result.set(j, input.get(i).unwrap());
                j += 1;
            }
            if i % 7 == 6 {
                result.set(j, input.get(i).unwrap());
                j += 1;
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
        assert_eq!(from_bitvec_to_string(Hamming.encode(BitVec::from_bytes(&[0b00110011]))), "10000111000011");
        assert_eq!(from_bitvec_to_string(Hamming.decode(Hamming.encode(BitVec::from_bytes(&[0b00110011]))).0), "00110011");
        assert_eq!(Hamming.decode(BitVec::from_bytes(&[0b10000111, 0b00001110, 0b00011100, 0b00111000, 0b01110000, 0b11100001, 0b11000011])).0, BitVec::from_bytes(&[51, 51, 51, 51]));
    }

    fn from_bitvec_to_string(input: BitVec) -> String {
        let mut result = String::new();
        for i in input.iter() {
            result.push(if i { '1' } else { '0' })
        }
        result
    }
}

