//! - `Data`
//!     * `Data coding:Coding compression:Compression length:int bytes:bytes = Data`, where `length` is # bits.

use super::*;

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Data {
    Data {
        coding: Coding,
        compression: Compression,
        length: i64,
        #[serde(with = "base64")]
        bytes: Vec<u8>,
    }
}

impl Data {
    pub fn from_bytes(bytes: &[u8], compression: Compression, coding: Coding) -> Result<Data, Error> {
        use ::compression::Compression;
        use ::coding::Coding;
        use ::bit_vec::BitVec;

        let compressor: Box<Compression<u8>> = compression.clone().into();
        let compressed: BitVec = compressor.compress(bytes).map_err(Error::Compression)?;
        let coder: Box<Coding> = coding.clone().into();
        let encoded: BitVec = coder.encode(compressed);

        let length = encoded.len() as i64;
        let bytes = encoded.to_bytes();

        Ok(Data::Data {
            coding,
            compression,
            length,
            bytes,
        })
    }

    /// decode and decompress bytes
    pub fn into_bytes(self) -> Result<Vec<u8>, Error> {
        use ::compression::Compression;
        use ::coding::Coding;
        use ::bit_vec::BitVec;

        match self {
            Data::Data {
                coding,
                compression,
                length,
                bytes,
            } => {
                let mut bits: BitVec = BitVec::from_bytes(&bytes);
                bits.truncate(length as usize);

                let decoder: Box<Coding> = coding.into();
                let (decoded, stats) = decoder.decode(bits);

                if stats.corrected < stats.detected {
                    Err(stats)?;
                }

                let compressor: Box<Compression<u8>> = compression.into();
                let decompressed = compressor.decompress(decoded)?;

                Ok(decompressed)
            }
        }
    }
}

/// chain errors
#[derive(Clone, Debug)]
pub enum Error {
    Decode(::coding::Stats),
    Compression(::compression::Error),
}

impl From<::coding::Stats> for Error {
    fn from(stats: ::coding::Stats) -> Self {
        Error::Decode(stats)
    }
}

impl From<::compression::Error> for Error {
    fn from(e: ::compression::Error) -> Self {
        Error::Compression(e)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;

    #[test]
    fn serde() {
        let bytes = b"rust";
        let data = Data::from_bytes(bytes, Compression::Rle, Coding::Parity).unwrap();
        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json, json!({
            "coding": "parity",
            "compression": "rle",
            "length": 50,
            "bytes": "9h5XqeZ6QA==",
        }));

        // serde_json can not work with untagged value
        let str = serde_json::to_string(&json).unwrap();
        // but is fine for str
        let data: Data = serde_json::from_str(&str).unwrap();
        let bytes = data.into_bytes().unwrap();
        assert_eq!(b"rust", bytes.as_slice());
    }
}