//! server methods:
//! - `login username:string = LoginResult`
//! - `getOnline = Online`
//! - `getUpdates username:string = Updates`
//! - `sendFile = FileId`
//! - `sendText from:Username to:Username payload:Data = Bool`
//! - `uploadFile from:Username to:Username file:FileMeta file_id:FileId payload:Data = Bool`
//! - `downloadFile file_id:FileId = Data`

use std::str;
use std::io::{self, Read};

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json;

use reqwest;

pub use super::types::base64;
pub use super::types::Username;
use super::types::GeneralAnswer;

pub mod login;
pub mod get_online;
pub mod get_updates;
pub mod send_file;
pub mod send_text;
pub mod upload_file;
pub mod download_file;

pub use self::login::Login;
pub use self::get_online::GetOnline;
pub use self::get_updates::GetUpdates;
pub use self::send_file::SendFile;
pub use self::send_text::SendText;
pub use self::upload_file::UploadFile;
pub use self::download_file::DownloadFile;


pub trait Method: Serialize + DeserializeOwned {
    type Answer: Serialize + DeserializeOwned + Clone + ::std::fmt::Debug;

    fn endpoint() -> &'static str;
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
    ServerError(String),
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

        let answer: GeneralAnswer<Self::Answer> = serde_json::from_str(body).map_err(ClientError::SerdeJson)?;

        debug!("json: {:?}", answer);

        match answer {
            GeneralAnswer::Ok(result) => Ok(result),
            GeneralAnswer::Err(description) => Err(ClientError::ServerError(description)),
        }
    }

    fn invoke_raw<T>(&self, target: &T) -> reqwest::Result<reqwest::Response>
        where T: Target
    {
        target.perform(M::endpoint(), &self)
    }
}