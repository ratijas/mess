//! - `LoginResult`
//!     * `LoginOk username:string = LoginResult`
//!     * `LoginErr = LoginResult`

#[derive(Debug)]
#[derive(Deserialize)]
#[serde(untagged)]
pub enum LoginResult {
    LoginOk { username: String },
    LoginErr,
}
