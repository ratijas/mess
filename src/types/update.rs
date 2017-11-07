//! - `Update`
//!     * `TextUpdate from:Username to:Username coding:string compression:string text:string = Update`
//!     * `FileUpdate from:Username to:Username coding:string compression:string file:FileMeta file_id:FileId = Update`

use super::Username;
use super::base64;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Update {
    TextUpdate {
        from: Username,
        to: Username,
        coding: String,
        compression: String,
        #[serde(with="base64")]
        text: Vec<u8>,
    }
}