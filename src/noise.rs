use rand;
use rand::Rng;

#[derive(Copy, Clone, Debug)]
pub enum NoiseLevel {
    /// 1% noise
    Noise001,
    /// 5% noise
    Noise005,
    /// 15% noise
    Noise015,
    /// 100% noise, flips all bits
    Noise100,
}

impl NoiseLevel {
    pub fn apply<I: Iterator>(self, iter: I) -> NoiseIter<I> {
        NoiseIter::<I> {
            it: iter,
            level: self.to_f64(),
            rng: rand::thread_rng(),
        }
    }

    pub fn to_f64(self) -> f64 {
        match self {
            NoiseLevel::Noise001 => 0.01,
            NoiseLevel::Noise005 => 0.05,
            NoiseLevel::Noise015 => 0.15,
            NoiseLevel::Noise100 => 1.00,
        }
    }
    pub fn to_str(self) -> &'static str {
        match self {
            NoiseLevel::Noise001 => "0.01",
            NoiseLevel::Noise005 => "0.05",
            NoiseLevel::Noise015 => "0.15",
            NoiseLevel::Noise100 => "1.00",
        }
    }
}

pub struct NoiseIter<I> {
    it: I,
    level: f64,
    rng: rand::ThreadRng,
}


impl<I: Iterator> Iterator for NoiseIter<I>
    where I::Item: ::std::ops::Not<Output=I::Item>
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.it.next().map(|i| {
            if self.rng.next_f64() < self.level {
                !i
            } else {
                i
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bit_vec::BitVec;

    #[test]
    fn flip_all() {
        let good = BitVec::from_elem(42, true);
        let bad = BitVec::from_elem(42, false);
        assert_eq!(bad, NoiseLevel::Noise100.apply(good.iter()).collect::<BitVec>());

        println!("{:?}", NoiseLevel::Noise001.apply(good.iter()).collect::<BitVec>());
        println!("{:?}", NoiseLevel::Noise005.apply(good.iter()).collect::<BitVec>());
        println!("{:?}", NoiseLevel::Noise015.apply(good.iter()).collect::<BitVec>());
        println!("{:?}", NoiseLevel::Noise100.apply(good.iter()).collect::<BitVec>());
    }
}