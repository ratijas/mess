//! Network routines

use reqwest;

use serde::Serialize;

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

    pub fn post<I: Serialize>(&self, method: &str, data: &I) -> reqwest::Result<reqwest::Response> {
        println!("json data: {:?}", ::serde_json::to_string(data));
        self.client.post(&self.url(method))
            .json(data)
            .send()
    }
}