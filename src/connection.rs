//! Network routines

use serde::Serialize;

use reqwest;

use algos::methods::Target;

#[derive(Clone)]
pub struct Connection {
    host: String,
    port: u16,
    client: reqwest::Client,
}

impl Connection {
    pub fn new<I: Into<String>>(host: I, port: u16) -> Self {
        let host = host.into();
        Connection {
            host,
            port,
            client: reqwest::Client::new(),
        }
    }

    fn url(&self, method: &str) -> String {
        format!("http://{}:{}/{}", self.host, self.port, method)
    }
}

impl Target for Connection {
    fn perform<I: Serialize>(&self, name: &str, data: &I) -> reqwest::Result<reqwest::Response> {
        self.client.post(&self.url(name))
            .json(data)
            .send()
    }
}