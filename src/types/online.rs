//! - `Online`
//!     * `online users:Vector<User> = Online`

use super::*;

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Online {
    Online { users: Vec<Username> }
}