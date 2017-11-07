/*
    На позиции степени 2 записываются repetition bite, и начиная с 2^i выделяются блоки размера 2^i через 1,
    для которых идет суммирование по модулю 2( xor ). 1 - для нечетного количества битов и 0 - для четного количества 1 в блоке.
*/

//use super::*;
use bit_vec::BitVec;

#[derive(Debug)]
pub struct Hamming;

impl Coding for Hamming {
    fn encode(&self, input: BitVec) -> BitVec {
        let len: f64 = input.iter().count();
        let result_len = len + len.log2().ceil() + 2; // because we start from 1 + log2(1) = 0

        let mut result = BitVec::from_elem(result_len, false);

        let mut j = 0;
        let mut st = 1;
        for i in 1..result_len {
            if i == st {
                j += 1;
                st *= 2;
            } else {
                result.set(i, input.get(j));
                j += 1;
            }
        }
        j = 1;
        for i in 0..len.log2().ceil() {
            let mut ans = 0;
            for k in j..result_len {
                for iter in 0..j {
                    if k + iter < result_len {
                        ans += result.get(k + iter);
                    }
                }
                k += j - 1;
            }
            result.set(j, ans % 2);
            j *= 2;
        }
        result
    }

    fn decode(&self, input: BitVec) -> (BitVec, Vec<usize>) {
        let len: f64 = input.iter().count();
        let result_len = len - len.log2().floor() - 2; // we return our message starts from 0
        let mut result = BitVec::from_elem(result_len, false);
        let mut j = 1;
        let mut diff = 0;
        for i in 0..len.log2().floor() {
            let mut ans = 0;
            for k in j..result_len {
                for iter in 0..j {
                    if k + iter < result_len {
                        ans += input.get(k + iter);
                    }
                }
                k += j;
            }
            if input.get(j) != ans % 2 {
                diff += j;
            }
            j *= 2;
        }
        input.set(j, (input.get(j) + 1) % 2); // if there are no errors, we change the element 0, which we ignore
        j = 0;
        let mut st = 1;
        for i in 1..result_len {
            if st == i {
                st *= 2;
            } else {
                result.set(j, input.get(i));
                j += 1;
            }
        }
        result
    }
}

