//! - `sendText from:Username to:Username coding:string compression:string text:string = Bool`

use super::Method;
use super::Username;
use super::base64;

#[derive(Serialize)]
pub struct SendText {
    pub from: Username,
    pub to: Username,
    pub coding: String,
    pub compression: String,
    #[serde(with = "base64")]
    pub text: Vec<u8>,
}

impl Method for SendText {
    type Answer = bool;

    fn endpoint(&self) -> &'static str {
        "sendText"
    }
}