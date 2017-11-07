//! - `Updates`
//!     * `Updates updates:Vector<Update> = Updates`

use super::Update;

#[derive(Debug)]
#[derive(Deserialize)]
#[serde(untagged)]
pub enum Updates {
    Updates { updates: Vec<Update> }
}