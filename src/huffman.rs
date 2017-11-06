//use super::*;
use std::collections::HashMap;
use std::hash::Hash;

use bit_vec::BitVec;

#[derive(Clone)]
pub struct Huffman<T> {
    pub events: HashMap<T, BitVec>,
}

pub type Probability = f64;

impl<T: Eq + Hash + Clone> Huffman<T> {
    pub fn new<I>(events: I) -> Self
        where
            I: IntoIterator<Item=(T, Probability)>,
    {
        let events: HashMap<T, Probability> = events.into_iter().collect();

        if !Huffman::check(&events) {
            panic!("probabilities are not in range [0..1] or does not sum up to 1")
        }

        Huffman { events: Huffman::huffman(events) }
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
            V: Hash + Eq + Clone,
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


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HuffmanError {
    UnexpectedData,
}


#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    // #[test]
    #[allow(unused)]
    fn into_iter() {
        let mut map = HashMap::new();

        for &(letter, p) in [
            ('A', 3_f64),
            ('B', 3_f64),
            ('C', 6_f64),
            ('D', 12_f64),
            ('E', 12_f64),
        ].iter()
            {
                map.insert(letter as u8, p.recip());
            }

        let h = Huffman::new(map);
        println!("{:?}", h.events);

        let s = b"ABADBECB";
        println!("source: {:?}", s);
        println!(
            "source as str: {}",
            String::from_utf8(Vec::from(&s[..])).unwrap()
        );
//        let encoded = dictionary_compression(h.events, s);
//        println!("{:?}", encoded);
    }
}
