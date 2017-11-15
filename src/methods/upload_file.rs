//! - `uploadFile from:Username to:Username file:FileMeta file_id:FileId payload:Data = Bool`
use super::Method;
use ::types;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct UploadFile {
    pub from: types::Username,
    pub to: types::Username,
    pub file: types::FileMeta,
    pub file_id: types::FileId,
    pub payload: types::Data,
}

impl Method for UploadFile {
    type Answer = bool;

    fn endpoint() -> &'static str {
        "uploadFile"
    }
}