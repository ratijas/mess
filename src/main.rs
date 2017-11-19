#![allow(unused_imports)]
#![feature(specialization)]

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


use tui::Terminal;
use tui::backend::{Backend, TermionBackend};
use tui::widgets::{border, Block, Paragraph, Widget, List, SelectableList};
use tui::layout::{Group, Rect, Size, Direction};
use tui::style::{Color, Modifier, Style};
use termion::input::TermRead;
use termion::event::{Event, Key};
use termion::screen::AlternateScreen;

// std
use std::io::{self, Write};
use std::thread;
use std::cell::RefCell;
use std::sync::{Arc, Weak};
use std::sync::mpsc::{channel, Sender, Receiver};

use algos::types::*;
use algos::methods::*;

use gui::*;
use gui::window::*;
use gui::main_view_controller::*;
use gui::list_view_controller::*;

pub mod client;
pub mod connection;
#[macro_use]
pub mod gui;

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

    // TODO
    root_view: Boxed<Window>,
}

pub enum AppEvent {
    Input(Event),
    Online(Online),
    Updates(Updates),
}

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
            // TODO
            root_view: Window::new(),
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
        let rx_username = app.on_username_change();
        thread::spawn(move || {
            let host = "0.0.0.0";
            let conn = connection::Connection::new(host, 3000);
            // first time blocking call to receiver
            let mut username = rx_username.recv().unwrap();

            loop {
                /*
                let updates = (methods::GetUpdates { username: username.clone() })
                    .invoke(&conn)
                    .unwrap();
                */
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
                // TODO

                // non-blocking check for changes
                match rx_username.try_recv() {
                    // discard updates, because they do not belong to current user anymore
                    Ok(u) => username = u,
                    _ => tx_event.send(AppEvent::Updates(updates)).unwrap(),
                }
            }
        });

        app
    }

    pub fn event_loop(mut self) -> Result<Terminal<TermionBackend>, io::Error> {
        loop {
            let size = self.t.borrow().size()?;
            if size != self.size {
                self.t.borrow_mut().resize(size)?;
                self.size = size;
            }
            (*self.root_view).borrow().render_on_termion(&mut *self.t.borrow_mut(), &self.size);

            let event = self.rx.recv().map_err(|_| io::Error::from(io::ErrorKind::Other))?;

            match event {
                AppEvent::Input(event) => {
                    self.handle_event(event);
                    if self.quit { break; }
                }
                AppEvent::Online(users) => {
                    match users {
                        Online::Online { users } => {
                            self.online = users;
                        }
                    }
                }
                AppEvent::Updates(updates) => {
                    match updates {
                        Updates::Updates { updates } => {
                            self.inbox.extend(updates);
                        }
                    }
                }
            }
        }
        Ok(self.t.into_inner())
    }

    pub fn on_username_change(&mut self) -> Receiver<Username> {
        let (tx, rx) = channel();
        self.username_listeners.push(tx);
        rx
    }

    pub fn username(&mut self, username: Username) {
        self.username = Some(username);
        for tx in self.username_listeners.iter() {
            tx.send(self.username.as_ref().unwrap().clone()).unwrap();
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        let mut controller: Boxed<ViewController> = (*self.root_view).borrow().active_child_view_controller().upgrade().unwrap();
        // bubble down
        loop {
            if (*controller).borrow_mut().handle_event(event.clone(), true) {
                return;
            }
            let child = (*controller).borrow().active_child_view_controller();
            if let Some(child) = child.upgrade() {
                controller = child;
            } else {
                break;
            }
        }
        // bubble up
        loop {
            if (*controller).borrow_mut().handle_event(event.clone(), false) {
                return;
            }
            let parent = (*controller).borrow().parent_view_controller();
            if let Some(parent) = parent.upgrade() {
                controller = parent;
            } else {
                break;
            }
        }

        match event {
            Event::Key(Key::Esc) => self.quit = true,
            _ => {}
        }

        /*
            Key::Ctrl('l') => {
                self.size = self.t.borrow().size()?;
                self.t.borrow_mut().resize(self.size)?;
                self.t.borrow_mut().clear()?;
            }
            Key::Backspace => {
                self.status.pop();
            }
            Key::Char(ch) => {
                self.status.push(ch);
            }
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
            key => {
                self.status = format!("key: {:?}", key);
            }
        }
        */
    }

    fn draw(&mut self) -> Result<(), io::Error> {
        // let mut cursor = termion::cursor::Goto::default();

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
                            .select_optional(index)
                            .highlight_style(Style::default().modifier(Modifier::Invert))
                            .block(Block::default().borders(border::ALL).title("Users online"))
                            .render(t, &chunks[0]);

                        (*self.root_view).borrow().render_on_termion(t, &chunks[1]);
                    });

                gui::text_field::TextField::default()
                    .text(&self.status)
                    .title("Status")
                    .render(t, &chunks[1]);
            });

        self.t.borrow_mut().draw()?;
        Ok(())
    }
}

fn main() {
    #[allow(unused)]
    let screen = AlternateScreen::from(io::stdout());

    let backend = TermionBackend::new().unwrap();
    let mut t = Terminal::new(backend).unwrap();
    t.hide_cursor().unwrap();


    let app = App::new(t);
    let _ = app.event_loop().unwrap();

    let backend = TermionBackend::new().unwrap();
    let mut t = Terminal::new(backend).unwrap();

    t.show_cursor().unwrap();
    t.clear().unwrap();

    // drop alternative screen, returning to normal

    drop(screen);

    /*

    let username: String = "".into();
    let client = client::Client::new(connection::Connection::new("0.0.0.0", 3000));
    // println!("username: {}", username);
    let res = (methods::Login { username: username.clone() }).invoke(&client.connection).unwrap();

    match res {
        LoginResult::LoginOk { username } => println!("logged in as {}", username),
        LoginResult::LoginErr => panic!("can not log in with this username!"),
    }

    let online: Online = (methods::Online {}).invoke(&client.connection).unwrap();

    println!("users online:");
    for (i, user) in match online { Online::Online { users } => users }.iter().enumerate() {
        println!("{}. {}", i + 1, user);
    }
    */

    // let peer = prompt("select peer to start chatting: ").unwrap();
    /*
    let username_clone = username.clone();
    let gui_clone = client.clone();
    let _ = ::std::thread::spawn(move || {
        loop {
            ::std::thread::sleep(::std::time::Duration::from_secs(1));
            let updates: Updates = (methods::GetUpdates { username: username_clone.clone() })
                .invoke(&gui_clone.connection)
                .unwrap();

            let updates: Vec<Update> = match updates {
                Updates::Updates { updates } => updates
            };

            for update in updates {
                match update {
                    Update::TextUpdate { from, text, .. } => {
                        let base = base64::decode(&text).unwrap();
                        let text = ::std::str::from_utf8(&base).unwrap();
                        println!();
                        println!("new message from {}: {}", from, text);
                    }
                    _ => {}
                }
            }
        }
    });
    */
    /*
    loop {
        let msg = prompt("type a message: ").unwrap();
        let sent = (methods::SendText {
            from: username.clone(),
            to: peer.clone(),
            coding: String::new(),
            compression: String::new(),
            text: msg.into_bytes(),
        }).invoke(&client.connection);
        println!("sent: {:?}", sent);
    }
    */
}

/*
fn prompt<I: ::std::fmt::Display>(prompt: I) -> Result<String, io::Error> {
    let mut answer = String::new();

    print!("{}", prompt);
    io::stdout().flush()?;
    io::stdin().read_line(&mut answer)?;
    answer.pop();
    Ok(answer)
}
*/