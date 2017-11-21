#![allow(unused_imports)]

// network
extern crate reqwest;
// serde
extern crate serde;
extern crate serde_json;
extern crate base64;
// gui
extern crate tui;
extern crate termion;
// algos
extern crate algos;
extern crate rand;

mod app;
mod error;
mod imports;
mod connection;
mod gui;
mod logo;
mod mode;


// std

use algos::types::*;
use algos::methods::*;

use imports::*;
use gui::*;

#[allow(unused)]
pub struct App {
    /// Logged in as `username`
    username: Option<Username>,
    /// internals of app events system
    username_listeners: Vec<Sender<Username>>,
    /// Result of Online RPC method call
    online: Vec<Username>,
    /// The user we are chatting with
    peer: Option<Username>,
    /// Inbox of current user
    inbox: Vec<Update>,
    /// status bar string
    status: String,
    /// terminal
    t: RefCell<Terminal<TermionBackend>>,
    /// size of terminal
    size: Rect,
    /// Quit flag
    quit: bool,
    /// Copy of sender for the Application Event bus
    tx: Sender<AppEvent>,
    /// Application Event bus
    rx: Receiver<AppEvent>,
}

pub enum AppEvent {
    Input(Event),
    Online(Online),
    Updates(Updates),
}

/*
#[allow(unused)]
impl App {
    fn new(t: Terminal<TermionBackend>) -> App {
        let (tx, rx) = channel::<AppEvent>();

        let mut app = App {
            username: None,
            username_listeners: Vec::new(),
            online: Vec::new(),
            peer: None,
            inbox: Vec::new(),
            status: String::new(),
            t: ::std::cell::RefCell::new(t),
            size: Default::default(),
            quit: false,
            tx,
            rx,
        };

        let tx_event = app.tx.clone();
        thread::spawn(move || {
            let stdin = io::stdin();
            for event in stdin.events() {
                let event = event.unwrap();
                tx_event.send(AppEvent::Input(event)).unwrap();
            }
        });

        let tx_event = app.tx.clone();
        thread::spawn(move || {
            let host = "0.0.0.0";
            let conn = connection::Connection::new(host, 3000);
            loop {
                // let online = (methods::Online {}).invoke(&conn).unwrap();
                let users = &[
                    "ivan",
                    "ratijas",
                    "nick",
                    "aidar",
                    "jack",
                    "summer",
                    "klyde",
                    "butters",
                    "dark pit",
                ];
                // TODO
                use rand::distributions::{IndependentSample, Range};
                use rand::Rng;

                let mut rng = rand::thread_rng();

                let range = Range::new(users.len() / 2, users.len());
                let amount = range.ind_sample(&mut rng);
                let mut online = rand::sample(&mut rng, users, amount);
                rng.shuffle(&mut online);
                let online = online.iter().map(|name| name.to_string()).collect();

                tx_event.send(AppEvent::Online(Online::Online { users: online })).unwrap();
                thread::sleep(std::time::Duration::from_secs(2));
            }
        });

        let tx_event = app.tx.clone();
        thread::spawn(move || {
            loop {
                *//*
                let updates = (methods::GetUpdates { username: username.clone() })
                    .invoke(&conn)
                    .unwrap();
                *//*
                let updates = Updates::Updates {
                    updates: vec![
                        Update::TextUpdate {
                            from: "mike".into(),
                            to: "ivan".into(),
                            payload: Data::from_bytes(
                                "wsup bro?!".as_bytes(),
                                Compression::Rle,
                                Coding::Hamming,
                            ).unwrap(),
                        },
                        Update::TextUpdate {
                            from: "nick".into(),
                            to: "ivan".into(),
                            payload: Data::from_bytes(
                                "hello, world!".as_bytes(),
                                Compression::Rle,
                                Coding::Hamming,
                            ).unwrap(),
                        }
                    ]
                };
                thread::sleep(::std::time::Duration::from_secs(5));
                tx_event.send(AppEvent::Updates(updates)).unwrap()
            }
        });

        app
    }

    pub fn handle_event(&mut self, event: Event) {
        *//*
            k @ Key::Up | k @ Key::Down => {
                let new_index = match self.peer {
                    Some(ref peer) => {
                        let index = self.online.iter().position(|online| online == peer);

                        match (k, index) {
                            (Key::Up, Some(index)) if index != 0 => index - 1,
                            (Key::Down, Some(index)) => {
                                (index + 1).min(self.online.len() - 1)
                            }
                            _ => 0
                        }
                    }
                    _ => 0,
                };
                self.peer = self.online.get(new_index).cloned();
            }
        }
        *//*
    }

    fn draw(&mut self) -> Result<(), io::Error> {
        Group::default()
            .direction(Direction::Vertical)
            .margin(0)
            .sizes(&[Size::Min(0), Size::Fixed(3)]) // status bar at the bottom
            .render(&mut *self.t.borrow_mut(), &self.size, |t, chunks| {
                // Main area
                Group::default()
                    .direction(Direction::Horizontal)
                    // online users list, dialog view
                    .sizes(&[Size::Percent(25), Size::Percent(75)])
                    .render(t, &chunks[0], |t, chunks| {
                        let index = self
                            .peer
                            .as_ref()
                            .and_then(|peer| self.online.iter().position(|item| item == peer));

                        SelectableList::default()
                            .items(&self.online)
                            // .select_optional(index)
                            .highlight_style(Style::default().modifier(Modifier::Invert))
                            .block(Block::default().borders(border::ALL).title("Users online"))
                            .render(t, &chunks[0]);
                    });
            });

        self.t.borrow_mut().draw()?;
        Ok(())
    }
}
*/

fn usage() -> ! {
    let name = env::args().next().unwrap();
    println!("usage: {} <username> <peer username>", name);
    ::std::process::exit(1);
}

fn main() {
    let mut args = env::args().skip(1);
    let me = match args.next() {
        Some(ok) => ok,
        None => usage(),
    };
    let peer = match args.next() {
        Some(ok) => ok,
        None => usage(),
    };

    let mut app = app::App::new(me, peer);

    use std::panic::{self, AssertUnwindSafe};
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        app.event_loop().unwrap();
    }));

    // return screen to normal
    drop(app);

    if let Err(e) = result {
        match e.downcast::<&str>() {
            Ok(s) => println!("{}", *s),
            Err(_) => println!("Unknown error"),
        }
        ::std::process::exit(1);
    }
}
