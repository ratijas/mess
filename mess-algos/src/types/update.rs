//! - `Update`
//!     * `TextUpdate from:Username to:Username payload:Data = Update`
//!     * `FileUpdate from:Username to:Username meta:FileMeta file_id:FileId = Update`

use super::*;

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Update {
    TextUpdate {
        from: Username,
        to: Username,
        payload: Data,
    },
    FileUpdate {
        from: Username,
        to: Username,
        meta: FileMeta,
        file_id: FileId,
    },
}