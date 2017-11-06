use super::*;

/// Parity-check code.
#[derive(Debug)]
pub struct Parity;

impl Code for Parity {
    fn encode (&self, input: BitVec) -> BitVec {
        let len = input.iter().count();
        let result_len = 5 * len / 4 + if len % 4 > 0 {len % 4 + 1} else {0};
        let mut result = BitVec::from_elem(result_len,false);
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

    fn decode (&self, input: BitVec) -> (BitVec, Vec<usize>) {
        let len = input.iter().count();
        let result_len = 4 * len / 5 + if len % 5 > 0 {len % 5 - 1} else {0};
        let mut result = BitVec::from_elem(result_len,false);
        let mut errors = Vec::new();

        let mut j = 0;
        let mut p = false;
        for i in 0..result_len {
            if let Some(bit) = input.get(j) {
                result.set(i, bit);
                j += 1;
                p = p ^ bit;
                if j % 5 == 4 {
                    if let Some(parity) = input.get(j) {
                        if p != parity {
                            errors.push((j - 1) / 5);
                        }
                    }
                    p = false;
                    j += 1;
                }
            }
        }
        (result, errors)
    }
}
