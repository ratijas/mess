//! - `DownloadedFile`
//!     * `File data:Data = DownloadedFile`
//!     * `EmptyFile = DownloadedFile`

use super::*;

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum DownloadedFile {
    File { data: Data },
    EmptyFile {},
}
