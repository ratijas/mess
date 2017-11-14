//! - `login username:string = LoginResult`
//! - `online = Online`
//! - `getUpdates username:string = Updates`
//! - `sendText from:Username to:Username coding:string compression:string text:string = Bool`
//! - `sendFile from:Username to:Username coding:string compression:string file:FileMeta = FileId`
//! - `uploadFile file_id:FileId bytes:bytes = Bool`
//! - `downloadFile file_id:FileId = bytes`

use std::str;
use std::io::{self, Read};

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json;

use reqwest;

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

pub trait Method: Serialize + DeserializeOwned {
    type Answer: Serialize + DeserializeOwned + ::std::fmt::Debug;

    fn endpoint(&self) -> &'static str;
}

/// Client side trait to invoke RPC method
pub trait ClientMethod: Method {
    fn invoke<T: Target>(&self, target: &T) -> Result<Self::Answer, ClientError>;
    fn invoke_raw<T: Target>(&self, target: &T) -> reqwest::Result<reqwest::Response>;
}

pub enum ClientError {
    ReqwestError(reqwest::Error),
    IoError(io::Error),
    Utf8Error(str::Utf8Error),
    SerdeJson(serde_json::Error),
    ServerError(Option<String>),
}

pub trait Target {
    fn perform<I: Serialize>(&self, name: &str, data: &I) -> reqwest::Result<reqwest::Response>;
}

/// Server side method to handle RPC method
///
/// Server side errors must be returned as a properly constructed `Answer`.
pub trait ServerMethod<Ctx>: Method {
    fn handle(self, context: &mut Ctx) -> Self::Answer;
}

#[derive(Debug)]
#[derive(Deserialize)]
struct GeneralAnswer<T: ::std::fmt::Debug> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
}

impl<M> ClientMethod for M
    where M: Method
{
    fn invoke<T>(&self, target: &T) -> Result<Self::Answer, ClientError>
        where T: Target
    {
        debug!("request: {}", serde_json::to_string(&self).unwrap());

        let mut res: reqwest::Response = self.invoke_raw(target).map_err(ClientError::ReqwestError)?;
        let mut body = Vec::new();
        res.read_to_end(&mut body).map_err(ClientError::IoError)?;
        let body = str::from_utf8(&body).map_err(ClientError::Utf8Error)?;

        debug!("response: {}", body);

        let json: GeneralAnswer<Self::Answer> = serde_json::from_str(body).map_err(ClientError::SerdeJson)?;

        debug!("json: {:?}", json);

        if !json.ok {
            return Err(ClientError::ServerError(json.description));
        }
        Ok(json.result.ok_or(ClientError::ServerError(None))?)
    }

    fn invoke_raw<T>(&self, target: &T) -> reqwest::Result<reqwest::Response>
        where T: Target
    {
        target.perform(self.endpoint(), &self)
    }
}