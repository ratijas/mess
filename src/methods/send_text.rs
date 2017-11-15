//! - `sendText from:Username to:Username payload:Data = Bool`

use super::Method;
use ::types;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct SendText {
    pub from: types::Username,
    pub to: types::Username,
    pub payload: types::Data,
}

impl Method for SendText {
    type Answer = bool;

    fn endpoint() -> &'static str {
        "sendText"
    }
}