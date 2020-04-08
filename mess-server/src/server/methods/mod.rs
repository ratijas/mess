mod login;
mod get_online;
mod get_updates;
mod send_file;
mod send_text;
mod upload_file;
mod download_file;

// for `use super::*;` inside submodules.
pub use ::algos::types::*;
pub use ::algos::methods::*;
pub use ::{App, User, File};
