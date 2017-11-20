use imports::*;
use algos::methods::ClientMethod;
use std::cell::RefCell;

#[derive(Eq, PartialEq)]
pub enum State {
    Initial,
    Error,
    LoggedIn,
    Exit,
}

#[allow(unused)]
pub enum AppEvent {
    Input(Event),
    Online(Online),
    Updates(Updates),

    SentText(SendText),
    SentFile(UploadFile),
    SendFailed,

    Log { message: String, error: bool },
}

pub struct App {
    state: State,
    /// user can't send new messages until he finishes with current.
    sending: bool,

    input: TextField,
    status: String,

    // users
    me: Username,
    peer: Username,

    // app internals
    events: (Sender<AppEvent>, Receiver<AppEvent>),

    // tui internals
    #[allow(unused)] screen: AlternateScreen<io::Stdout>,
    t: RefCell<Terminal<TermionBackend>>,
    size: Rect,
}

impl App {
    pub fn new(me: Username, peer: Username) -> App {
        let screen = AlternateScreen::from(io::stdout());
        let backend = TermionBackend::new().unwrap();
        let mut t = Terminal::new(backend).unwrap();
        t.hide_cursor().unwrap();

        App {
            state: State::Initial,
            sending: false,

            input: TextField::new(),
            status: String::new(),

            me,
            peer,

            events: channel(),

            screen,
            t: RefCell::new(t),
            size: Default::default(),
        }
    }

    pub fn error(&mut self, msg: String) {
        self.state = State::Error;
        self.status = msg;
    }

    fn set_up(&mut self) {
        let tx = self.events.0.clone();
        thread::spawn(move || {
            let stdin = io::stdin();
            for event in stdin.events() {
                let event = event.unwrap();
                tx.send(AppEvent::Input(event)).unwrap();
            }
        });
    }

    fn login(&mut self) {
        let method = Login { username: self.me.clone() };
        match method.invoke(&Connection::default()) {
            Ok(answer) => {
                match answer {
                    LoginResult::LoginOk { username } => {
                        self.me = username;
                        self.state = State::LoggedIn;
                        self.status = format!("Logged in as {}", self.me);
                    }
                    LoginResult::LoginErr => {
                        let msg = format!("Login error: username \"{}\" can not be used", &self.me);
                        self.error(msg);
                        self.me.clear();
                    }
                }
            }
            Err(e) => {
                self.error(format!("{:?}", e));
            }
        }
    }

    pub fn event_loop(&mut self) -> io::Result<()> {
        self.set_up();
        self.login();

        while self.state != State::Exit {
            self.resize_maybe()?;

            {
                let t = &mut *self.t.borrow_mut();
                self.draw(t)?;
                // drop t
            }

            let event = self.events.1.recv().map_err(|_| io::Error::from(io::ErrorKind::Other))?;
            self.handle_app_event(event);
        }

        Ok(())
    }

    fn resize_maybe(&mut self) -> io::Result<()> {
        let t = &mut *self.t.borrow_mut();

        let size = t.size()?;
        if size != self.size {
            t.resize(size)?;
            self.size = size;
        }
        Ok(())
    }

    fn draw(&self, t: &mut Terminal<TermionBackend>) -> io::Result<()> {
        Group::default()
            .direction(Direction::Vertical)
            .margin(0)
            .sizes(&[Size::Min(0), Size::Fixed(2), Size::Fixed(2)]) // status bar at the bottom
            .render(t, &self.size, |t, chunks| {
                Block::default()
                    .borders(border::ALL)
                    .render(t, &chunks[0]);

                //                Group::default()
                //                    .direction(Direction::Vertical)
                //                    .margin(1)
                //                    .sizes(&[Size::Min(0), Size::Fixed(1), Size::Fixed(2)])
                //                    .render(t, &chunks[0], |t, chunks| {});

                LineEdit::default()
                    .label("Text")
                    .text(&self.input.buffer)
                    .cursor(self.input.cursor)
                    .focus(!self.sending)
                    .render(t, &chunks[1]);

                StatusBar::default()
                    .message(&self.status)
                    .error(self.state == State::Error)
                    .render(t, &chunks[2]);
            });
        t.draw()?;
        Ok(())
    }

    fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Input(ev) => self.handle_input(ev),
            AppEvent::Log { message, error } => {
                if error {
                    self.state = State::Error;
                } else {
                    self.state = State::LoggedIn;
                }
                self.status = message;
            }
            AppEvent::SentText(msg) => {
                //self.messages.push(msg);

                self.input.reset();
                self.sending = false;
            }
            AppEvent::SendFailed => {
                self.sending = false;
            }
            _ => {}
        }
    }

    fn handle_input(&mut self, event: Event) {
        match event {
            Event::Key(Key::Esc) => {
                if !self.status.is_empty() {
                    self.status.clear();
                } else {
                    self.state = State::Exit;
                }
            }
            Event::Key(Key::Char('\n')) => {
                self.send();
            }
            Event::Key(Key::Ctrl('l')) => { /* redraw */ }
            event => {
                if !self.sending {
                    self.input.handle_event(event);
                }
            }
        }
    }

    fn send(&mut self) {
        self.sending = true;

        if self.input.buffer.len() == 0 {
            self.sending = false;
            self.events.0.send(AppEvent::SendFailed).unwrap();
            error(&self.events.0, "Message is empty");
            return;
        }

        let content = self.input.buffer.clone();
        let me = self.me.clone();
        let peer = self.peer.clone();

        let tx = self.events.0.clone();
        tx.send(AppEvent::Log { message: "sending message".into(), error: false }).unwrap();
        thread::spawn(move || {
            thread::sleep(::std::time::Duration::from_millis(500));
            let method = SendText {
                from: me,
                to: peer,
                payload: Data::from_bytes(
                    content.as_bytes(),
                    Compression::Rle,
                    Coding::Hamming,
                ).unwrap(),
            };
            info(&tx, "Send message: compressed and encoded");

            let result = method.invoke(&Connection::default());
            thread::sleep(::std::time::Duration::from_millis(500));

            match result {
                Ok(answer) => {
                    if answer == false {
                        error(&tx, "Send message: server error");
                        tx.send(AppEvent::SendFailed).unwrap();
                    } else {
                        info(&tx, "Send message: done");
                        tx.send(AppEvent::SentText(method)).unwrap();
                    }
                }
                Err(e) => {
                    error(&tx, format!("Send message: Error: {:?}", e));
                    tx.send(AppEvent::SendFailed).unwrap();
                }
            }
        });
    }
}

fn info<I: Into<String>>(tx: &Sender<AppEvent>, message: I) {
    let message = message.into();
    tx.send(AppEvent::Log { message, error: false }).unwrap();
}

fn error<I: Into<String>>(tx: &Sender<AppEvent>, message: I) {
    let message = message.into();
    tx.send(AppEvent::Log { message, error: true }).unwrap();
}

impl Drop for App {
    fn drop(&mut self) {
        let t = &mut *self.t.borrow_mut();

        t.show_cursor().unwrap();
        t.clear().unwrap();
    }
}