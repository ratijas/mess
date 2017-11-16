//! server constructors, types:
//! - `int = i64`
//! - `Bool`
//!     * `True = Bool`
//!     * `False = Bool`
//! - `bytes`
//!     * b64 encoded string
//!
//! - `Vector<T>` is vector of elements of type T
//!
//! - `LoginResult`
//!     * `LoginOk username:string = LoginResult`
//!     * `LoginErr = LoginResult`
//!
//! - `Online`
//!     * `Online users:Vector<Username> = Online`
//!
//! - `Updates`
//!     * `Updates updates:Vector<Update> = Updates`
//!
//! - `Update`
//!     * `TextUpdate from:Username to:Username payload:Data = Update`
//!     * `FileUpdate from:Username to:Username file:FileMeta file_id:FileId = Update`
//!
//! - `FileMeta`:
//!     * `FileMeta name:string size:int mime:string = FileMeta`, where `size` is # bytes.
//!
//! - `FileId`
//!     * `FileId file_id:int = FileId`
//!
//! - `Data`
//!     * `Data coding:Coding compression:Compression length:int bytes:bytes = Data`, where `length` is # bits.
//!
//! - `Compression`
//!     * `Rle = Compression`
//! // TODO
//!     * `Huffman events:??? = Compression`
//!     * `Shannon events:??? = Compression`
//!
//! - `Coding`
//!     * `Hamming = Coding`
//!     * `Parity = Coding`
//!     * `R3 = Coding`
//!     * `R5 = Coding`


pub mod login_result;
pub mod online;
pub mod update;
pub mod updates;
pub mod file_meta;
pub mod file_id;
pub mod data;
pub mod coding;
pub mod compression;

pub use self::login_result::LoginResult;
pub use self::online::Online;
pub use self::update::Update;
pub use self::updates::Updates;
pub use self::file_meta::FileMeta;
pub use self::file_id::FileId;
pub use self::data::Data;
pub use self::coding::Coding;
pub use self::compression::Compression;

pub type Username = String;

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub enum GeneralAnswer<T>
    where T: Clone + ::std::fmt::Debug
{
    #[serde(rename = "result")]
    Ok(T),
    #[serde(rename = "description")]
    Err(String),
}

pub mod base64 {
    extern crate base64;

    use serde::{Serializer, de, Deserialize, Deserializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&base64::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
        where D: Deserializer<'de>
    {
        let s = <&str>::deserialize(deserializer)?;
        base64::decode(s).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;

    #[test]
    fn serde() {
        let err: GeneralAnswer<()> = GeneralAnswer::Err("bad stuff".into());
        let str = serde_json::to_string(&err).unwrap();

        assert_eq!(r#"{"description":"bad stuff"}"#, str);

        let answer: GeneralAnswer<()> = serde_json::from_str(&str).unwrap();
        match answer {
            GeneralAnswer::Ok(_) => unreachable!(),
            GeneralAnswer::Err(s) => assert_eq!(s, "bad stuff"),
        }
    }
}