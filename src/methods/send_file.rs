//! - `sendFile = FileId`

use super::Method;
use ::types;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct SendFile {}


impl Method for SendFile {
    type Answer = types::FileId;

    fn endpoint() -> &'static str {
        "sendFile"
    }
}