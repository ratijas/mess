//! Hamming (4/7)


use super::{Coding, Stats};
use bit_vec::BitVec;

#[derive(Debug)]
pub struct Hamming;

impl Coding for Hamming {
    fn encode(&self, input: BitVec) -> BitVec {
        let len = input.len();
        let result_len = (len / 4) * 7 + if len % 4 == 1 { 2 } else if len % 4 > 1 { 3 } else { 0 } + len % 4;

        let mut result = BitVec::from_elem(result_len, false);

        for i in 0..(len + 3) / 4 {
            let b0 = input.get(0 + i * 4).unwrap();
            let b1 = if 1 + i * 4 < len { input.get(1 + i * 4).unwrap() } else { false };
            let b2 = if 2 + i * 4 < len { input.get(2 + i * 4).unwrap() } else { false };
            let b3 = if 3 + i * 4 < len { input.get(3 + i * 4).unwrap() } else { false };

            result.set(0 + i * 7, b0 ^ b1 ^ b3);//
            result.set(1 + i * 7, b0 ^ b2 ^ b3);//
            result.set(2 + i * 7, b0);
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
        let result_len = (len / 7) * 4 + if len % 7 == 3 { 1 } else if len % 7 > 3 { len % 7 - 3 } else { 0 };

        let mut result = BitVec::from_elem(result_len, false);

        let mut diff;
        for i in 0..(len + 6) / 7 {
            diff = 0;
            let mut b = [false; 7];
            for j in 0..7 {
                b[j] = if let Some(bit) = input.get(j + i * 7) { bit } else { false };
            }
            if b[2] ^ b[4] ^ b[6] != b[0] { diff += 1 };
            if b[2] ^ b[5] ^ b[6] != b[1] { diff += 2 };
            if b[4] ^ b[5] ^ b[6] != b[3] { diff += 4 };
            if diff > 0 {
                diff -= 1;
                input.set(diff + 7 * i, !b[diff]);
                stats.detected += 1;
                stats.corrected += 1;
            }
        }

        for i in 0..result_len {
            let j = if i % 4 == 0 { (i / 4) * 7 + 2 } else { (i / 4) * 7 + i % 4 + 3 };
            result.set(i, input.get(j).unwrap());
        }

        (result, stats)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(bv2str(Hamming.encode(str2bv("00110011"))), "10000111000011");
        assert_eq!(bv2str(Hamming.encode(str2bv("0011001101"))), "1000011100001110011");
        assert_eq!(bv2str(Hamming.encode(str2bv("0"))), "000");
        assert_eq!(bv2str(Hamming.encode(str2bv("01"))), "10011");
        assert_eq!(bv2str(Hamming.encode(str2bv("010"))), "100110");
        assert_eq!(bv2str(Hamming.encode(str2bv("0101"))), "0100101");
    }

    #[test]
    fn test_decode() {
        //without flips
        assert_eq!(bv2str(Hamming.decode(str2bv("10000111000011")).0), "00110011");
        assert_eq!(bv2str(Hamming.decode(str2bv("1000011100001110011")).0), "0011001101");
        assert_eq!(bv2str(Hamming.decode(str2bv("000")).0), "0");
        assert_eq!(bv2str(Hamming.decode(str2bv("10011")).0), "01");
        assert_eq!(bv2str(Hamming.decode(str2bv("100110")).0), "010");
        assert_eq!(bv2str(Hamming.decode(str2bv("0100101")).0), "0101");
        //with one flip per block
        assert_eq!(bv2str(Hamming.decode(str2bv("10100111000001")).0), "00110011");
        assert_eq!(bv2str(Hamming.decode(str2bv("1001011000001110111")).0), "0011001101");
        assert_eq!(bv2str(Hamming.decode(str2bv("010")).0), "0");
        assert_eq!(bv2str(Hamming.decode(str2bv("10001")).0), "01");
        assert_eq!(bv2str(Hamming.decode(str2bv("000110")).0), "010");
        assert_eq!(bv2str(Hamming.decode(str2bv("0110101")).0), "0101");
    }

    fn bv2str(input: BitVec) -> String {
        let mut result = String::new();
        for i in input.iter() {
            result.push(if i { '1' } else { '0' })
        }
        result
    }

    fn str2bv(input: &str) -> BitVec {
        let mut result = BitVec::from_elem(input.len(), false);
        let mut i = 0;
        for ch in input.chars() {
            if ch == '1' { result.set(i, true) }
            i += 1;
        }
        result
    }
}

