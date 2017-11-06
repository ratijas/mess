//! - `User`
//!     * `user username:string = User`
//!
//! - `LoginResult`
//!     * `LoginOk username:string = LoginResult`
//!     * `LoginErr = LoginResult`
//!
//! - `Online`
//!     * `online users:Vector<User> = Online`

pub mod login_result;
pub mod online;
pub mod user;

pub use self::login_result::LoginResult;
pub use self::online::Online;
pub use self::user::User;
