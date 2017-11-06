use super::*;

/// Repetition5 code.
#[derive(Debug)]
pub struct Repetition5;

impl Code for Repetition5 {
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

    fn decode(&self, input: BitVec) -> (BitVec, Vec<usize>) {
        let len = input.iter().count();
        let result_len = len / 5;
        let mut result = BitVec::from_elem(result_len, false);
        let mut j = 0;
        for i in 0..result_len {
            let mut temp = 0;
            while j / 5 < i + 1 {
                if let Some(bit) = input.get(j) {
                    if bit {
                        temp += 1;
                    }
                }
                j+=1;
            }
            if temp >= 3 {
                result.set(i, true);
            }
        }
        (result, Vec::new())
    }
}
