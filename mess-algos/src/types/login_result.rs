//! - `LoginResult`
//!     * `LoginOk username:string = LoginResult`
//!     * `LoginErr = LoginResult`

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum LoginResult {
    LoginOk { username: String },
    LoginErr,
}