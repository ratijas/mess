//! Network routines

use imports::*;

use serde::Serialize;
use reqwest;

#[derive(Clone)]
pub struct Connection {
    host: String,
    port: u16,
    client: reqwest::blocking::Client,
}

impl Connection {
    pub fn new<I: Into<String>>(host: I, port: u16) -> Self {
        let host = host.into();
        Connection {
            host,
            port,
            client: reqwest::blocking::ClientBuilder::new().timeout(Duration::from_secs(5)).build().unwrap(),
        }
    }

    fn url(&self, method: &str) -> String {
        format!("http://{}:{}/{}", self.host, self.port, method)
    }
}

impl Default for Connection {
    fn default() -> Self {
        let host = env::var("MESS_HOST").unwrap_or("0.0.0.0".to_string());
        Connection::new(host, 3000)
    }
}

impl Target for Connection {
    fn perform<I: Serialize>(&self, name: &str, data: &I) -> reqwest::Result<reqwest::blocking::Response> {
        self.client.post(&self.url(name))
            .json(data)
            .send()
    }
}