pub mod client;
pub mod connection;

extern crate pancurses;
extern crate reqwest;

extern crate serde;
#[allow(unused)]
#[macro_use]
extern crate serde_derive;
#[allow(unused)]
#[macro_use]
extern crate serde_json;
extern crate base64;

extern crate algos;

pub use algos::{types, methods};

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

    let peer = prompt("select peer to start chatting: ").unwrap();

    let username_clone = username.clone();
    let gui_clone = gui.clone();
    let _ = ::std::thread::spawn(move || {
        loop {
            ::std::thread::sleep(::std::time::Duration::from_secs(1));
            let updates: types::Updates = (methods::GetUpdates { username: username_clone.clone() }).invoke(&gui_clone.connection).unwrap();

            let updates: Vec<types::Update> = match updates {
                types::Updates::Updates { updates } => updates
            };

            for update in updates {
                match update {
                    types::Update::TextUpdate {
                        from, text, ..
                    } => {
                        let base = base64::decode(&text).unwrap();
                        let text = ::std::str::from_utf8(&base).unwrap();
                        println!();
                        println!("new message from {}: {}", from, text);
                    }
                }
            }
        }
    });

    loop {
        let msg = prompt("type a message: ").unwrap();
        let sent = (methods::SendText {
            from: username.clone(),
            to: peer.clone(),
            coding: String::new(),
            compression: String::new(),
            text: msg.into_bytes(),
        }).invoke(&gui.connection);
        println!("sent: {:?}", sent);
    }
}

fn prompt<I: ::std::fmt::Display>(prompt: I) -> Result<String, io::Error> {
    let mut answer = String::new();

    print!("{}", prompt);
    io::stdout().flush()?;
    io::stdin().read_line(&mut answer)?;
    answer.pop();
    Ok(answer)
}