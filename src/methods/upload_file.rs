//! - `uploadFile from:Username to:Username meta:FileMeta file_id:FileId payload:Data = Bool`
use super::Method;
use ::types;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct UploadFile {
    pub from: types::Username,
    pub to: types::Username,
    pub meta: types::FileMeta,
    pub file_id: types::FileId,
    pub payload: types::Data,
}

impl Method for UploadFile {
    type Answer = bool;

    fn endpoint() -> &'static str {
        "uploadFile"
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;

    #[test]
    fn serde() {
        let method = UploadFile {
            from: "daniel".into(),
            to: "frank".into(),
            meta: types::FileMeta::FileMeta {
                name: "Best language".into(),
                size: 4,
                mime: "text/plain".into(),
            },
            file_id: types::FileId::FileId(0),
            payload: types::Data::from_bytes(
                b"rust",
                types::Compression::Rle,
                types::Coding::Parity,
            ).unwrap()
        };

        let str = serde_json::to_string(&method).unwrap();

        println!("method uploadFile: {}", str);

        let de: UploadFile = serde_json::from_str(&str).unwrap();

        println!("de: {:?}", de);
    }
}