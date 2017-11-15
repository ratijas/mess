//! - `Update`
//!     * `TextUpdate from:Username to:Username payload:Data = Update`
//!     * `FileUpdate from:Username to:Username file:FileMeta file_id:FileId payload:Data = Update`

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
        file: FileMeta,
        file_id: FileId,
        payload: Data,
    },
}