//! - `FileId`
//!     * `FileId file_id:int = FileId`

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum FileId {
    FileId { file_id: i64 }
}