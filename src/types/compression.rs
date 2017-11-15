//! - `Compression`
//!     * `Rle = Compression`
//! // TODO
//!     * `Huffman events:??? = Compression`
//!     * `Shannon events:??? = Compression`

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Compression {
    Rle,
    // TODO
    Huffman,
    Shannon,
}

impl Into<Box<::compression::Compression<u8>>> for Compression {
    fn into(self) -> Box<::compression::Compression<u8>> {
        match self {
            Compression::Rle => Box::new(::compression::rle::Rle),
            // TODO
            _ => panic!(),
        }
    }
}

impl From<::compression::rle::Rle> for Compression {
    fn from(_: ::compression::rle::Rle) -> Self {
        Compression::Rle
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;

    #[test]
    fn ser() {
        let json = serde_json::to_string(&Compression::Rle).unwrap();
        assert_eq!(r#""rle""#, json);
    }

    #[test]
    fn de() {
        let compression = serde_json::from_str(r#""rle""#).unwrap();
        match compression {
            Compression::Rle => assert!(true),
            _ => unreachable!(),
        }
    }
}