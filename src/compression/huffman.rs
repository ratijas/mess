use std::collections::HashMap;
use std::hash::Hash;

use bit_vec::BitVec;

use super::{Compression, Decompression};

#[derive(Clone)]
pub struct Huffman<T> {
    pub events: HashMap<T, BitVec>,
    /// reverse map
    pub codes: HashMap<BitVec, T>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HuffmanError {
    UnexpectedMoreData,
}

pub type Probability = f64;

impl<T: Eq + Hash + Clone> Huffman<T> {
    fn new(events: HashMap<T, BitVec>) -> Self {
        let codes = events
            .iter()
            .map(|(k, v)| {
                (v.clone(), k.clone())
            })
            .collect();
        Huffman { events, codes }
    }

    pub fn with_probabilities<I>(events: I) -> Self
        where I: IntoIterator<Item=(T, Probability)>
    {
        let events: HashMap<T, Probability> = events.into_iter().collect();

        if !Huffman::check(&events) {
            panic!("probabilities are not in range [0..1] or does not sum up to 1")
        }

        Huffman::new(Huffman::huffman(events))
    }

    pub fn with_map(map: HashMap<T, BitVec>) -> Self {
        Huffman::new(map)
    }

    pub fn optimal_for(input: &[T]) -> Self {
        let mut frequency: HashMap<T, usize> = HashMap::new();

        let mut total: usize = 0;
        for i in input.iter() {
            let ent = frequency.entry(i.clone()).or_insert(0usize);
            *ent += 1;
            total += 1;
        }

        Huffman::with_probabilities(
            frequency
                .into_iter()
                .map(|(i, count)| {
                    (i, (count as Probability) / (total as Probability))
                }))
    }

    fn check(pairs: &HashMap<T, Probability>) -> bool {
        let mut sum = 0f64;

        for p in pairs.values() {
            // p shall be in range [0..1]
            if p.is_sign_negative() || *p > 1.0 {
                return false;
            }
            sum += *p;
        }
        // sum of probabilities be exactly 1
        if !((sum - 1.0).abs() < 1e-10) {
            return false;
        }
        // we are good to go...
        true
    }

    /// construct optimal map
    fn huffman(p: HashMap<T, Probability>) -> HashMap<T, BitVec> {
        let raw = p.into_iter().map(|(k, f)| (vec![k], f)).collect();
        Huffman::huffman_raw(raw)
            .into_iter()
            .map(|(keys, code)| match keys.len() {
                1 => (keys.get(0).unwrap().clone(), code),
                _ => unreachable!(),
            })
            .collect()
    }

    fn huffman_raw(p: HashMap<Vec<T>, Probability>) -> HashMap<Vec<T>, BitVec> {
        if p.len() == 2 {
            return p.into_iter()
                    .map(|(k, _)| k)
                    .zip(vec![
                        BitVec::from_elem(1, false),
                        BitVec::from_elem(1, true),
                    ])
                    .collect();
        } else {
            let mut p_prime = p.clone();
            let (e1, e2) = Huffman::<T>::lowest_prob_pair(&p);
            let p1 = p_prime.remove(e1).unwrap();
            let p2 = p_prime.remove(e2).unwrap();
            let e: Vec<T> = {
                let mut e = Vec::new();
                e.append(&mut e1.clone());
                e.append(&mut e2.clone());
                e
            };
            p_prime.insert(e.clone(), p1 + p2);

            let mut c = Huffman::huffman_raw(p_prime);
            let ce1e2 = c.remove(&e).unwrap();
            c.insert(e1.clone(), {
                let mut v = ce1e2.clone();
                v.push(false);
                v
            });
            c.insert(e2.clone(), {
                let mut v = ce1e2.clone();
                v.push(true);
                v
            });
            c
        }
    }

    fn lowest_prob_pair<'a, V>(x: &'a HashMap<V, Probability>) -> (&'a V, &'a V)
        where
            V: Hash + Eq + Clone
    {
        let first: &'a V = x.iter()
                            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                            .unwrap()
                            .0;
        let second: &'a V = x.iter()
                             .filter(|&(k, _)| k != first)
                             .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                             .unwrap()
                             .0;
        (first, second)
    }
}

impl<T> Compression<T> for Huffman<T>
    where T: Eq + Hash + Clone
{
    type Error = HuffmanError;

    fn compress(&self, input: &[T]) -> Result<BitVec, Self::Error> {
        let mut output = BitVec::new();

        for i in input {
            match self.events.get(i) {
                Some(code) => output.extend(code),
                None => return Err(HuffmanError::UnexpectedMoreData),
            }
        }
        Ok(output)
    }
}

impl<T> Decompression<T> for Huffman<T>
    where T: Eq + Hash + Clone
{
    type Error = HuffmanError;

    fn decompress(&self, input: BitVec) -> Result<Vec<T>, Self::Error> {
        let mut offset: usize = 0;
        let mut output: Vec<T> = Vec::new();

        while offset < input.len() {
            let mut slice = BitVec::new();

            while let None = self.codes.get(&slice) {
                match input.get(offset) {
                    None => return Err(HuffmanError::UnexpectedMoreData),
                    Some(bit) => {
                        slice.push(bit);
                        offset += 1;
                    }
                }
            }

            let code = self.codes.get(&slice).unwrap();
            output.push((*code).clone());
        }

        Ok(output)
    }
}

#[cfg(test)]
mod test {
    //! how the hell we're supposed to test it?
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test() {
        let mut map = HashMap::new();

        for &(letter, p) in [
            ('A', 3_f64),
            ('B', 3_f64),
            ('C', 6_f64),
            ('D', 12_f64),
            ('E', 12_f64),
        ].iter() {
            map.insert(letter as u8, p.recip());
        }

        let h = Huffman::with_probabilities(map);

        let mut lengths = h.events.iter().map(|(_, code)| code.len()).collect::<Vec<_>>();
        lengths.sort();
        assert_eq!(&[1, 2, 3, 4, 4], &*lengths);

        let s = b"ABADBECB";
        let encoded = h.compress(s);
        assert!(encoded.is_ok());
    }

    #[test]
    fn optimal() {
        let h: Huffman<u8> = Huffman::optimal_for(b"01110-1111");
        assert_eq!(1, h.events.get(&('1' as u8)).unwrap().len());
        assert_eq!(2, h.events.get(&('0' as u8)).unwrap().len());
        assert_eq!(2, h.events.get(&('-' as u8)).unwrap().len());
    }

    #[test]
    fn reverse() {
        let s = "hello, world!";
        let h: Huffman<u8> = Huffman::optimal_for(s.as_bytes());
        let vec = h.compress(s.as_bytes());

        assert!(vec.is_ok());
        assert_eq!(s.as_bytes(), &*h.decompress(vec.unwrap()).unwrap());
    }
}
