//! - `login username:string = LoginResult`
//! - `online = Online`
//! - `getUpdates username:string = Updates`
//! - `sendText from:Username to:Username coding:string compression:string text:string = Bool`
//! - `sendFile from:Username to:Username coding:string compression:string file:FileMeta = FileId`
//! - `uploadFile file_id:FileId bytes:bytes = Bool`
//! - `downloadFile file_id:FileId = bytes`

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use reqwest;

use super::connection::Connection;

pub mod login;
pub mod online;
pub mod get_updates;

pub use self::login::Login;
pub use self::online::Online;
pub use self::get_updates::GetUpdates;


pub trait Method: Serialize {
    type Answer: DeserializeOwned;

    fn endpoint(&self) -> &'static str;

    fn invoke(&self, conn: &Connection) -> Result<Self::Answer, ()> {
        let res = conn.post(self.endpoint(), &self);

        // println!("response: {:?}", res);

        let mut value: Value = res
            .map_err(drop)?
            .json()
            .map_err(drop)?;

        // println!("value: {:?}", value);

        let obj = value.as_object_mut().ok_or(())?;
        let ok = obj.remove("ok").ok_or(())?;
        if !ok.as_bool().unwrap_or(false) {
            return Err(());
        }
        let result = obj.remove("result").ok_or(())?;
        // println!("result: {:?}", result);
        ::serde_json::from_value::<Self::Answer>(result).map_err(drop)
    }

    fn invoke_raw(&self, conn: &Connection) -> reqwest::Result<reqwest::Response> {
        conn.post(self.endpoint(), &self)
    }
}