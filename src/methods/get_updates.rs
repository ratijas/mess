//! - `getUpdates username:string = Updates`

use super::Method;
use ::types::Updates;


#[derive(Serialize)]
pub struct GetUpdates {
    pub username: String,
}

impl Method for GetUpdates {
    type Answer = Updates;

    fn endpoint(&self) -> &'static str {
        "getUpdates"
    }
}