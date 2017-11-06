pub mod client;
pub mod connection;

pub mod methods;
pub mod types;

extern crate pancurses;
extern crate reqwest;

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;


use std::io::{self, Write};

use methods::Method;

fn main() {
    let gui = client::Client::new(connection::Connection::new("0.0.0.0", 3000));

    let mut username = String::new();
    io::stdout().write_all(b"username: ").unwrap();
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut username).unwrap();
    let username = username.trim_right().to_string();

    let res = methods::Login { username }.invoke(&gui.connection);

    println!("result: {:?}", res);

    let online = methods::Online {}.invoke(&gui.connection);

    println!("online: {:?}", online);

}
