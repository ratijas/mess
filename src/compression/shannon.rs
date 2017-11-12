use std::collections::HashMap;
use std::hash::Hash;

use bit_vec::BitVec;

use super::Compression;
use super::huffman::Huffman;

pub type Probability = f64;

pub struct ShannonFano<T> {
    pub huffman: Huffman<T>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShannonError {
    UnexpectedMoreData,
}

impl<T> ShannonFano<T>
    where T: Eq + Hash + Clone
{
    pub fn with_probabilities(events: HashMap<T, Probability>) -> ShannonFano<T> {
        let mut output = HashMap::new();
        let mut pairs = events.into_iter().collect::<Vec<(T, Probability)>>();
        pairs.sort_by(|&(_, p1), &(_, p2)| p1.partial_cmp(&p2).unwrap());

        add_bit(&mut output, &pairs, true);
        ShannonFano {
            huffman: Huffman::with_map(output)
        }
    }

    pub fn optimal_for(input: &[T]) -> Self {
        let mut frequency: HashMap<T, usize> = HashMap::new();

        let mut total: usize = 0;
        for i in input {
            let ent = frequency.entry(i.clone()).or_insert(0usize);
            *ent += 1;
            total += 1;
        }

        ShannonFano::with_probabilities(
            frequency
                .into_iter()
                .map(|(i, count)| {
                    (i, (count as Probability) / (total as Probability))
                })
                .collect()
        )
    }
}


impl<T> Compression<T> for ShannonFano<T>
    where T: Eq + Hash + Clone
{
    type Error = ShannonError;

    fn compress(&self, input: &[T]) -> Result<BitVec, Self::Error> {
        self.huffman.compress(input).map_err(|_| ShannonError::UnexpectedMoreData)
    }

    fn decompress(&self, input: BitVec) -> Result<Vec<T>, Self::Error> {
        self.huffman.decompress(input).map_err(|_| ShannonError::UnexpectedMoreData)
    }
}


fn add_bit<T>(result: &mut HashMap<T, BitVec>, pairs: &[(T, Probability)], upper: bool)
    where T: Hash + Eq + Clone {
    let len = pairs.len();

    if result.is_empty() {
        for &(ref t, _) in pairs {
            result.insert(t.clone(), BitVec::new());
        }
    } else {
        for &(ref t, _) in pairs {
            let bits = result.get_mut(t).unwrap();
            bits.push(upper);
        }
    }

    if len >= 2 {
        let mut separator: usize = 0;
        let total: Probability = pairs.iter().map(|&(_, p)| p).sum();
        let median = total / 2f64;
        let mut sum = 0 as Probability;

        for (i, &(_, p)) in pairs.iter().enumerate() {
            if sum + p > median {
                if (sum - median).abs() > (sum + p - median).abs() {
                    // sum += p;
                    separator = i + 1;
                }
                break;
            }
            sum += p;
            separator = i + 1;
        }
        add_bit(result, &pairs[..separator], true);
        add_bit(result, &pairs[separator..], false);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_shannon() {
        let mut map = HashMap::new();

        for &(letter, p) in [
            (1, 0.36),
            (2, 0.18),
            (3, 0.18),
            (4, 0.12),
            (5, 0.09),
            (6, 0.07),
        ].iter() {
            map.insert(letter, p);
        }

        let shannon = ShannonFano::with_probabilities(map);

        println!("shannon: {:?}", shannon.huffman.events);

        let mut lengths = shannon.huffman.events.iter().map(|(_, code)| code.len()).collect::<Vec<_>>();
        lengths.sort();
        assert_eq!(&[2, 2, 2, 3, 4, 4], &*lengths);

        let msg = &[1, 5, 2, 4, 3, 2, 5, 1];
        let encoded = shannon.compress(msg);
        assert!(encoded.is_ok());
    }

    #[test]
    fn reverse() {
        let s = "hello, world!";
        let h: ShannonFano<u8> = ShannonFano::optimal_for(s.as_bytes());
        let vec = h.compress(s.as_bytes());

        assert!(vec.is_ok());
        assert_eq!(s.as_bytes(), &*h.decompress(vec.unwrap()).unwrap());
    }
}