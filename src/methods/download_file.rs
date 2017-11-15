//! - `downloadFile file_id:FileId = Data`

use super::Method;
use ::types;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct DownloadFile {
    pub file_id: types::FileId,
}

impl Method for DownloadFile {
    type Answer = types::Data;

    fn endpoint() -> &'static str {
        "downloadFile"
    }
}