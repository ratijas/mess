//! - `FileId`
//!     * `FileId file_id:int = FileId`

#[derive(Clone, Debug)]
#[derive(Hash, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum FileId {
    FileId(i64)
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;

    #[test]
    fn serde() {
        let fid = FileId::FileId(42);
        let str = serde_json::to_string(&fid).unwrap();

        assert_eq!("42", &str);

        let fid_de: FileId = serde_json::from_str(&str).unwrap();

        assert_eq!(fid, fid_de);
        assert_eq!(42, match fid_de { FileId::FileId(id) => id });
    }
}