use std::ops::Neg;

use bit_vec::BitVec;
use void::Void;

use super::{Compression, Decompression};

/// Run-length encoding (RLE).
///
/// This type of compression oriented on data with lots of repetitive symbols in
/// a row. It suites best for spare planes, like in [Conway's "Game of Life"].
///
/// Input byte stream is encoded as pairs `(count, symbol)` for each repetitive
/// `symbol` byte where `count` is signed 8-bit integer.
///
/// * When `count >= 1` then exactly that much `symbol`s must be in source message.
/// * When `count < 0` then `-count` is the amount of subsequent raw bytes which must be just copied.
/// * It is an error when `count == 0`.
/// * It is acceptable if `count == 1`, albeit inefficient.
/// * As a consequence of these rules, `-1` and `1` for `count` means the same.
///
/// [Conway's "Game of Life"]: http://www.conwaylife.com/
#[derive(Debug)]
pub struct Rle;

impl Compression<u8> for Rle {
    type Error = Void;

    fn compress(&self, input: &[u8]) -> Result<BitVec, Self::Error> {
        /*
        states:
        count == 0, initial
        count == 1, new => buffering, incrementing count
        count >= 1, same => incrementing count
        count > 1, new => flush
        count == max, any => flush
        */

        fn flush(out: &mut Vec<u8>, count: i8, ch: u8) {
            out.push(count as u8);
            out.push(ch);
        };

        fn flush_buffer(out: &mut Vec<u8>, buffer: &[u8]) {
            out.push((buffer.len() as isize).neg() as u8);
            out.extend(buffer);
        }

        enum State {
            Initial,
            FirstNew { new: u8 },
            /// always `count >= 2`.
            ManySame { last: u8, count: i8 },
            /// `buffer` always contains at least 2 bytes.
            /// `count` indicates how many times the last byte in `buffer` has been repeated,
            /// always either 0 or 1, where 0 means there are unique characters at the tail,
            /// and 1 means there is a double.
            ManyDifferent { buffer: Vec<u8>, count: i8 },
        }

        let mut state = State::Initial;
        let mut out = Vec::new();  // output buffer.  working with bytes.  converting to bits at the end.

        for ch in input.into_iter().cloned() {
            match state {
                State::Initial => state = State::FirstNew { new: ch },
                State::FirstNew { new: last } => {
                    if last == ch {
                        state = State::ManySame { last, count: 2 }
                    } else {
                        state = State::ManyDifferent { buffer: vec![last, ch], count: 0 }
                    }
                }
                State::ManySame { last, count } => {
                    if last == ch {
                        if count == ::std::i8::MAX {
                            flush(&mut out, count, last);
                            state = State::FirstNew { new: ch };
                        } else {
                            state = State::ManySame { last, count: count + 1 };
                        }
                    } else {
                        if count == 2 {
                            state = State::ManyDifferent { buffer: vec![last, last, ch], count: 0 };
                        } else {
                            flush(&mut out, count, last);
                            state = State::FirstNew { new: ch };
                        }
                    }
                }
                State::ManyDifferent { mut buffer, count } => {
                    match buffer.len() {
                        len if len > (-(::std::i8::MIN as isize)) as usize => unreachable!(),
                        len if len == (-(::std::i8::MIN as isize)) as usize => {
                            flush_buffer(&mut out, &buffer);
                            state = State::FirstNew { new: ch };
                        }
                        _ => {
                            let last = *buffer.last().unwrap();
                            if last != ch {
                                buffer.push(ch);
                                state = State::ManyDifferent { buffer, count: 0 }
                            } else {
                                // don't hurry up flushing the buffer.  maybe it just 2 bytes repeated in the middle of random data.
                                if count == 0 {
                                    buffer.push(ch);
                                    state = State::ManyDifferent { buffer, count: 1 }
                                } else if count == 1 {
                                    flush_buffer(&mut out, &buffer[..buffer.len() - 2]);
                                    state = State::ManySame { last, count: 3 };
                                } else {
                                    unreachable!();
                                }
                            }
                        }
                    }
                }
            }
        }

        match state {
            State::FirstNew { new } => flush(&mut out, 1i8, new),
            State::ManySame { last, count } => flush(&mut out, count, last),
            State::ManyDifferent { buffer, .. } => flush_buffer(&mut out, &buffer),
            _ => {}
        }

        Ok(BitVec::from_bytes(&*out))
    }
}

impl Decompression<u8> for Rle {
    type Error = RleError;

    fn decompress(&self, input: BitVec) -> Result<Vec<u8>, RleError> {
        enum State {
            Initial,
            /// always `count > 0`.
            Same { count: i8 },
            /// negated counter shows how many characters left to copy.
            Different { count: i8 },
        }

        let mut state = State::Initial;
        let mut out = Vec::new();

        for ch in input.to_bytes().into_iter() {
            match state {
                State::Initial => {
                    match ch as i8 {
                        count if count > 0 => state = State::Same { count },
                        count if count < 0 => state = State::Different { count },
                        0 | _ => return Err(RleError::ZeroRepetition),
                    }
                }
                State::Same { count } => {
                    for _ in 0..count {
                        out.push(ch);
                    }
                    state = State::Initial;
                }
                State::Different { count } if count < 0 => {
                    out.push(ch);
                    state = match count {
                        -1 => State::Initial,
                        count => State::Different { count: count + 1 }
                    };
                }
                _ => unreachable!(),
            }
        }

        match state {
            State::Initial => Ok(out),
            _ => Err(RleError::ExpectedMoreData),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RleError {
    ExpectedMoreData,
    ZeroRepetition
}

#[cfg(test)]
mod test {
    use super::*;
    use std::iter::once;

    fn test_compression(value: &[u8]) {
        let compressed: BitVec = Rle.compress(value).unwrap();
        assert!(compressed.to_bytes().len() <= 1 + value.len());
        let decompressed = Rle.decompress(compressed);
        assert!(decompressed.is_ok());
        assert_eq!(decompressed.unwrap(), value);
    }

    #[test]
    fn rle_compress() {
        assert_eq!(&*Rle.compress(b"r").unwrap().to_bytes(), [1u8, 'r' as u8]);
        assert_eq!(&*Rle.compress(b"rr").unwrap().to_bytes(), [2u8, 'r' as u8]);
        assert_eq!(&*Rle.compress(b"rrr").unwrap().to_bytes(), [3u8, 'r' as u8]);
        assert_eq!(&*Rle.compress(b"ab").unwrap().to_bytes(), [-2i8 as u8, 'a' as u8, 'b' as u8]);
        assert_eq!(&*Rle.compress(b"abb").unwrap().to_bytes(), [-3i8 as u8, 'a' as u8, 'b' as u8, 'b' as u8]);
        assert_eq!(&*Rle.compress(b"abbb").unwrap().to_bytes(), [-1i8 as u8, 'a' as u8, 3u8, 'b' as u8]);
        assert_eq!(&*Rle.compress(b"abbbb").unwrap().to_bytes(), [-1i8 as u8, 'a' as u8, 4u8, 'b' as u8]);
        assert_eq!(&*Rle.compress(b"abc").unwrap().to_bytes(), [-3i8 as u8, 'a' as u8, 'b' as u8, 'c' as u8]);
        assert_eq!(&*Rle.compress(b"abbc").unwrap().to_bytes(), [-4i8 as u8, 'a' as u8, 'b' as u8, 'b' as u8, 'c' as u8]);
        assert_eq!(&*Rle.compress(b"abbcc").unwrap().to_bytes(), [-5i8 as u8, 'a' as u8, 'b' as u8, 'b' as u8, 'c' as u8, 'c' as u8]);

        //              h                 e                 l        l        o
        //          01101000          01100101          01101100 01101100 01101111
        //     1        h        1        e        2        l        1        o
        // 00000001 01101000 00000001 01100101 00000010 01101100 00000001 01101111 // +3, worst case
        //    -2        h                 e        2        l        1        o
        // 11111110 01101000          01100101 11111101 01101100 00000001 01101111 // +2, sort of fine
        //    -5        h                 e                 l        l        o
        // 11111011 01101000          01100101          01101100 01101100 01101111 // +1, perfect
    }

    #[test]
    fn edge_cases() {
        assert_eq!(&*Rle.compress("a".repeat(::std::i8::MAX as usize).as_bytes()).unwrap().to_bytes(),
                   [::std::i8::MAX as u8, 'a' as u8]);

        assert_eq!(&*Rle.compress("a".repeat(1 + ::std::i8::MAX as usize).as_bytes()).unwrap().to_bytes(),
                   [::std::i8::MAX as u8, 'a' as u8, 1u8, 'a' as u8]);

        let min = (-(::std::i8::MIN as isize)) as usize;
        let seq: Vec<u8> = (0..min).map(|i| i as u8).collect();

        let expected: Vec<u8> = once(min as u8).chain(seq.clone()).collect();
        assert_eq!(&*Rle.compress(&seq).unwrap().to_bytes(),
                   &*expected);

        let seq: Vec<u8> = (0..min + 1).map(|i| i as u8).collect();

        let expected: Vec<u8> = once(min as u8)
            .chain(seq.clone().into_iter().take(min))
            .chain(once(1u8))
            .chain(once(min as u8))
            .collect();
        assert_eq!(&*Rle.compress(&seq).unwrap().to_bytes(),
                   &*expected);

        assert_eq!(Rle.compress(b"").unwrap().to_bytes(), &[0u8; 0]);
    }

    #[test]
    fn rle_decompress() {
        let aaa = b"aaa";
        test_compression(aaa);
        let hello = b"hello";
        test_compression(hello);
    }

    #[test]
    fn rle_decompress_errors() {
        // 0xfb is -5
        assert_eq!(Rle.decompress(BitVec::from_bytes(b"\xfbabcd")), Err(RleError::ExpectedMoreData));
        assert_eq!(Rle.decompress(BitVec::from_bytes(b"\x04 \x02.\x07")), Err(RleError::ExpectedMoreData));
        assert_eq!(Rle.decompress(BitVec::from_bytes(b"\x04 \x00.\x04 ")), Err(RleError::ZeroRepetition));
    }
}
