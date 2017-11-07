//! - `Online`
//!     * `online users:Vector<User> = Online`


use super::Username;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Online {
    Online { users: Vec<Username> }
}