use super::*;

/// Repetition3 code.
#[derive(Debug)]
pub struct Repetition3;

impl Code for Repetition3 {
    fn encode(&self, input: BitVec) -> BitVec {
        let len = input.iter().count();
        let result_len = len * 3;
        let mut result = BitVec::from_elem(result_len, false);
        for i in 0..len {
            if let Some(bit) = input.get(i) {
                let j = i * 3;
                result.set(j, bit);
                result.set(j + 1, bit);
                result.set(j + 2, bit);
            }
        }
        result
    }

    fn decode(&self, input: BitVec) -> (BitVec, Vec<usize>) {
        let len = input.iter().count();
        let result_len = len / 3;
        let mut result = BitVec::from_elem(result_len, false);
        let mut j = 0;
        for i in 0..result_len {
            let mut temp = 0;
            while j / 3 < i + 1 {
                if let Some(bit) = input.get(j) {
                    if bit {
                        temp += 1;
                    }
                }
                j+=1;
            }
            if temp >= 2 {
                result.set(i, true);
            }
        }
        (result, Vec::new())
    }
}