//! - `Updates`
//!     * `Updates updates:Vector<Update> = Updates`

use super::*;

#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Updates {
    Updates { updates: Vec<Update> }
}