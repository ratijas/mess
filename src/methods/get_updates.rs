//! - `getUpdates username:string = Updates`

use super::Method;
use ::types;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct GetUpdates {
    pub username: String,
}

impl Method for GetUpdates {
    type Answer = types::Updates;

    fn endpoint(&self) -> &'static str {
        "getUpdates"
    }
}