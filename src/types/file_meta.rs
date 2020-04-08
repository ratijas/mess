//! - `FileMeta`:
//!     * `FileMeta name:string size:int mime:string = FileMeta`, where `size` is # bytes.

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum FileMeta {
    FileMeta {
        name: String,
        size: i64,
        mime: String
    }
}