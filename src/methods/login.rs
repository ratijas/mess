//! - `login username:string = LoginResult`

use super::Method;
use ::types::LoginResult;


#[derive(Serialize)]
pub struct Login {
    pub username: String,
}

impl Method for Login {
    type Answer = LoginResult;

    fn endpoint(&self) -> &'static str {
        "login"
    }
}