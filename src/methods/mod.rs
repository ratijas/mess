//! - `login username:string = LoginResult`
//! - `online = Online`
//! - `getUpdates username:string = Updates`
//! - `sendText from:Username to:Username coding:string compression:string text:string = Bool`
//! - `sendFile from:Username to:Username coding:string compression:string file:FileMeta = FileId`
//! - `uploadFile file_id:FileId bytes:bytes = Bool`
//! - `downloadFile file_id:FileId = bytes`

use std::str;
use std::io::Read;

use serde::Serialize;
use serde::de::DeserializeOwned;

use reqwest;

use super::connection::Connection;

pub mod login;
pub mod online;
pub mod get_updates;
pub mod send_text;

pub use self::login::Login;
pub use self::online::Online;
pub use self::get_updates::GetUpdates;
pub use self::send_text::SendText;

pub use super::types::base64;
pub use super::types::Username;

pub trait Method: Serialize {
    type Answer: DeserializeOwned + ::std::fmt::Debug;

    fn endpoint(&self) -> &'static str;

    fn invoke(&self, conn: &Connection) -> Result<Self::Answer, ()> {
        // println!("request: {}", ::serde_json::to_string(&self).unwrap());
        let res = conn.post(self.endpoint(), &self);

        let mut body = Vec::new();
        res.map_err(drop)?.read_to_end(&mut body).map_err(drop)?;
        let body = str::from_utf8(&body).map_err(drop)?;

        // println!("response: {}", body);

        let json: GeneralAnswer<Self::Answer> = ::serde_json::from_str(body).map_err(drop)?;

        // println!("json: {:?}", json);

        if !json.ok {
            return Err(());
        }
        Ok(json.result.ok_or(())?)
    }

    fn invoke_raw(&self, conn: &Connection) -> reqwest::Result<reqwest::Response> {
        conn.post(self.endpoint(), &self)
    }
}

#[derive(Debug)]
#[derive(Deserialize)]
struct GeneralAnswer<T: ::std::fmt::Debug> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
}