pub mod login_result;
pub mod online;
pub mod update;
pub mod updates;

pub use self::login_result::LoginResult;
pub use self::online::Online;
pub use self::update::Update;
pub use self::updates::Updates;

pub type Username = String;


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