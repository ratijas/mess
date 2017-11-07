pub mod client;
pub mod connection;

pub mod methods;
pub mod types;

extern crate pancurses;
extern crate reqwest;

extern crate serde;
#[macro_use]
extern crate serde_derive;
// #[macro_use]
extern crate serde_json;


use std::io::{self, Write};

use methods::Method;

fn main() {
    let gui = client::Client::new(connection::Connection::new("0.0.0.0", 3000));

    let username = prompt("username: ").unwrap();

    let res = methods::Login { username: username.clone() }.invoke(&gui.connection).unwrap();

    match res {
        types::LoginResult::LoginOk { username } => println!("logged in as {}", username),
        types::LoginResult::LoginErr => panic!("can not log in with this username!"),
    }

    let online: types::Online = (methods::Online {}).invoke(&gui.connection).unwrap();

    println!("users online:");
    for (i, user) in match online { types::Online::Online { users } => users }.iter().enumerate() {
        println!("{}. {}", i + 1, user);
    }

    let _peer = prompt("select peer to start chatting: ").unwrap();

    let upd = types::Update::TextUpdate {
        from: username.clone(),
        to: String::new(),
        coding: String::new(),
        compression: String::new(),
        text: "hello".as_bytes().to_vec(),
    };
    let ser = serde_json::to_string(&upd).unwrap();
    println!("text update: {:?}", ser);

    let upds: types::Updates = (methods::GetUpdates { username: username.clone() }).invoke(&gui.connection).unwrap();
    println!("updates: {:?}", upds);

//    let msg = prompt("type a message: ");

}

fn prompt<I: ::std::fmt::Display>(prompt: I) -> Result<String, io::Error> {
    let mut answer = String::new();

    print!("{}", prompt);
    io::stdout().flush()?;
    io::stdin().read_line(&mut answer)?;
    answer.pop();
    Ok(answer)
}