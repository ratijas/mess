//! - `Online`
//!     * `online users:Vector<User> = Online`

use super::User;


#[derive(Debug)]
#[derive(Deserialize)]
#[serde(untagged)]
pub enum Online {
    Online { users: Vec<String> }
}
