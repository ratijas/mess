//! - `online username:string = Online`

use super::Method;
use ::types;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Online {}


impl Method for Online {
    type Answer = types::Online;

    fn endpoint(&self) -> &'static str {
        "online"
    }
}