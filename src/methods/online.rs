//! - `online username:string = Online`

use super::Method;
use ::types;

#[derive(Serialize)]
pub struct Online {}


impl Method for Online {
    type Answer = types::Online;

    fn endpoint(&self) -> &'static str {
        "online"
    }
}
