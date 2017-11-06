//! - `User`
//!     * `user username:string = User`


#[derive(Debug)]
#[derive(Deserialize)]
#[serde(untagged)]
pub enum User {
    User { username: String }
}
