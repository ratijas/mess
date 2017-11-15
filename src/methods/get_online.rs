//! - `online username:string = Online`

use super::Method;
use ::types;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct GetOnline {}


impl Method for GetOnline {
    type Answer = types::Online;

    fn endpoint(&self) -> &'static str {
        "getOnline"
    }
}