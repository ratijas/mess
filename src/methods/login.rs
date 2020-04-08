//! - `login username:string = LoginResult`

use super::Method;
use ::types;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Login {
    pub username: String,
}

impl Method for Login {
    type Answer = types::LoginResult;

    fn endpoint() -> &'static str {
        "login"
    }
}