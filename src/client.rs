//! Console client for Mess

use ::connection::Connection;


#[derive(Clone)]
pub struct Client {
    pub connection: Connection,
}

impl Client {
    pub fn new(connection: Connection) -> Client {
        Client { connection }
    }
}